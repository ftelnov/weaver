#![allow(dead_code)]
//! Various runtimes for hyper
use std::{
    fmt::Display,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};

use futures_io::{AsyncRead, AsyncWrite};
use pin_project_lite::pin_project;
use tarantool::{
    fiber::{self},
    network::client::tcp::TcpStream,
};

#[derive(Clone)]
pub struct TarantoolHyperExecutor {
    child_name: String,
}

impl TarantoolHyperExecutor {
    pub fn new(server_name: impl Display) -> Self {
        Self {
            child_name: format!("{server_name}-child"),
        }
    }
}

impl<F> hyper::rt::Executor<F> for TarantoolHyperExecutor
where
    F: std::future::Future + 'static,
    F::Output: 'static,
{
    fn execute(&self, fut: F) {
        fiber::Builder::new()
            .name(&self.child_name)
            .func_async(fut)
            .defer_non_joinable()
            .expect("weaver can't create helper fiber");
    }
}

pin_project! {
    pub struct TarantoolAsyncIO {
        #[pin]
        stream: TcpStream,
    }
}

impl TarantoolAsyncIO {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl hyper::rt::Read for TarantoolAsyncIO {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let len = match AsyncRead::poll_read(self.project().stream, cx, unsafe {
            uninit_as_u8_slice(buf.as_mut())
        }) {
            Poll::Ready(Ok(len)) => len,
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            Poll::Pending => return Poll::Pending,
        };

        unsafe {
            buf.advance(len);
        }

        Poll::Ready(Ok(()))
    }
}

impl hyper::rt::Write for TarantoolAsyncIO {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        AsyncWrite::poll_write(self.project().stream, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        AsyncWrite::poll_flush(self.project().stream, cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        AsyncWrite::poll_close(self.project().stream, cx)
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<Result<usize, std::io::Error>> {
        AsyncWrite::poll_write_vectored(self.project().stream, cx, bufs)
    }
}

unsafe fn uninit_as_u8_slice(buf: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len())
}
