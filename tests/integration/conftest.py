import subprocess

import pytest

from tests.integration.utils import wait_server_alive


@pytest.fixture(scope="module")
def test_app():
    endpoint = "http://localhost:18989"
    process = subprocess.Popen(
        [
            "tarantool-runner",
            "run",
            "-p",
            "./target/debug/libintegration_suite_app.so",
            "-e",
            "run_server",
        ]
    )
    wait_server_alive("localhost", "18989")
    yield endpoint
    process.kill()
