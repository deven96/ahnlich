import os
import random
import signal
import socket
import subprocess
import time

import pytest

import config
from internals import protocol


def is_port_occupied(port, host="127.0.0.1") -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(1)
        result = sock.connect_ex((host, port))
        return result == 0


@pytest.fixture
def base_protocol():
    host = os.environ.get("AHNLICH_DB_HOST", "127.0.0.1")
    port = int(os.environ.get("AHNLICH_DB_PORT", 1369))
    print(host, port)
    return protocol.AhnlichProtocol(address=host, port=port)


@pytest.fixture
def random_port():
    port = random.randint(5000, 8000)
    return port


@pytest.fixture
def spin_up_ahnlich_db(random_port):
    port = random_port
    command = f"cargo run --bin ahnlich-db run --port {port}".split(" ")
    process = subprocess.Popen(args=command, cwd=config.AHNLISH_BIN_DIR)
    time.sleep(2)
    assert is_port_occupied(port)
    print("Server is spun up")
    yield port
    # cleanup
    print("Server is shutting down")
    os.kill(process.pid, signal.SIGINT)
    # wait for process to clean up
    process.wait(5)
