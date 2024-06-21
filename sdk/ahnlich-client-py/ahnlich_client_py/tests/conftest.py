import os
import random
import signal
import socket
import subprocess
import time

import numpy as np
import pytest

from ahnlich_client_py import client, config
from ahnlich_client_py.internals import query
from ahnlich_client_py.internals import serde_types as st


def is_port_occupied(port, host="127.0.0.1") -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(1)
        result = sock.connect_ex((host, port))
        return result == 0


@pytest.fixture(scope="module")
def db_client():
    host = os.environ.get("AHNLICH_DB_HOST", "127.0.0.1")
    port = int(os.environ.get("AHNLICH_DB_PORT", 1369))
    timeout_sec = float(os.environ.get("AHNLICH_DB_CLIENT_TIMEOUT", 5.0))
    return client.AhnlichDBClient(address=host, port=port, timeout_sec=timeout_sec)


@pytest.fixture
def random_port():
    port = random.randint(5000, 8000)
    return port


@pytest.fixture
def spin_up_ahnlich_db(random_port):
    port = random_port
    command = f"cargo run --bin ahnlich-db run --port {port}".split(" ")
    process = subprocess.Popen(args=command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(port):
        time.sleep(0.2)
    yield port
    # cleanup
    os.kill(process.pid, signal.SIGINT)
    # wait for process to clean up
    process.wait(5)


@pytest.fixture(scope="module")
def module_scopped_ahnlich_db():
    port = 8001
    command = f"cargo run --bin ahnlich-db run --port {port}".split(" ")
    process = subprocess.Popen(args=command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(port):
        time.sleep(0.2)
    yield port
    # cleanup
    os.kill(process.pid, signal.SIGINT)
    # wait for process to clean up
    process.wait(5)


@pytest.fixture
def store_key():
    sample_array = np.array([1.0, 2.0, 3.0, 4.0, 5.0], dtype=np.float32)
    dimensions = (st.uint64(sample_array.shape[0]),)
    store_key = query.Array(v=st.uint8(1), dim=dimensions, data=sample_array.tolist())
    return store_key


@pytest.fixture
def store_value():
    store_value = dict(job="sorcerer")
    return store_value
