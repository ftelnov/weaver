import subprocess
import pytest

from tests.integration.utils import wait_server_alive

@pytest.fixture(scope="module")
def test_app():
    endpoint = "http://localhost:18989"
    process = subprocess.Popen(["tarantool-runner", "run", "-p", "./target/debug/libapp.so", "-e", "run_server"])
    wait_server_alive(endpoint)
    yield endpoint
    (stdout, stderr) = process.communicate()
    print(f"Stdout of halted process: {stdout}, stderr: {stderr}")
