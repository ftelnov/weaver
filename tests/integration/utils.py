import asyncio
import logging
import socket
import threading
import time
from contextlib import closing

import httpx


def wait_server_alive(server):
    begin = time.time()
    timeout = 10
    client = httpx.Client()
    while (time.time() - begin) < timeout:
        try:
            client.get(server, timeout=3)
            return
        except Exception:
            pass
    raise Exception(f"Server at {server} didn't spin up properly")
