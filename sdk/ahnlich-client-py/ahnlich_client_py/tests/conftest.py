import os
import random
import signal
import socket
import subprocess
import time

import pytest

from ahnlich_client_py import clients, config, db_query
from ahnlich_client_py.libs import create_store_key


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
    conn = clients.AhnlichDBClient(
        address=host, port=port, connect_timeout_sec=timeout_sec
    )
    yield conn
    conn.cleanup()


@pytest.fixture(scope="module")
def ai_client():
    host = os.environ.get("AHNLICH_AI_HOST", "127.0.0.1")
    port = int(os.environ.get("AHNLICH_AI_PORT", 8000))
    timeout_sec = float(os.environ.get("AHNLICH_AI_CLIENT_TIMEOUT", 5.0))
    conn = clients.AhnlichAIClient(
        address=host, port=port, connect_timeout_sec=timeout_sec
    )
    yield conn
    conn.cleanup()


@pytest.fixture
def db_random_port():
    port = random.randint(5000, 7999)
    return port


@pytest.fixture
def ai_random_port():
    port = random.randint(8009, 9000)
    return port


@pytest.fixture
def spin_up_ahnlich_db(db_random_port):
    port = db_random_port
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
    sample_array = [1.0, 2.0, 3.0, 4.0, 5.0]
    return create_store_key(sample_array)


@pytest.fixture
def store_value():
    return dict(job=db_query.MetadataValue__RawString("sorcerer"))


@pytest.fixture
def aiproxy_default_ahnlich_db():
    port = 1369
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
def spin_up_ahnlich_ai(ai_random_port, aiproxy_default_ahnlich_db):
    port = ai_random_port
    command = f"cargo run --bin ahnlich-ai run --supported-models dalle3,llama3 --port {port}".split(
        " "
    )
    process = subprocess.Popen(args=command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(port):
        time.sleep(0.2)
    yield port
    # cleanup
    os.kill(process.pid, signal.SIGINT)
    # wait for process to clean up
    process.wait(5)


@pytest.fixture(scope="module")
def module_scopped_ahnlich_ai():
    port = 9001
    command = f"cargo run --bin ahnlich-ai run --supported-models dalle3,llama3 --port {port}".split(
        " "
    )
    process = subprocess.Popen(args=command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(port):
        time.sleep(0.2)
    yield port
    # cleanup
    os.kill(process.pid, signal.SIGINT)
    # wait for process to clean up
    process.wait(5)
