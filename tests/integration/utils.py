import socket
import time


def wait_server_alive(host, port):
    begin = time.time()
    timeout = 10
    while (time.time() - begin) < timeout:
        try:
            s = socket.create_connection((host, port), timeout=timeout)
            s.close()
            return
        except Exception as exc:
            print(f"Unable to connect to {host}:{port}: {exc}")
            time.sleep(1)
            pass
    raise Exception(f"Server at {host}:{port} didn't spin up properly")
