import asyncio
import os
import random
import signal
import socket
import subprocess
import time

import pytest

from ahnlich_client_py import config


def is_port_occupied(port, host="127.0.0.1") -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(1)
        result = sock.connect_ex((host, port))
        return result == 0


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
    command = f"cargo run --bin ahnlich-ai run --supported-models all-minilm-l6-v2,resnet-50 --port {port}".split(
        " "
    )
    process = subprocess.Popen(args=command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(port):
        time.sleep(0.2)
    time.sleep(1)
    yield port
    # cleanup
    os.kill(process.pid, signal.SIGINT)
    # wait for process to clean up
    process.wait(5)


@pytest.fixture(scope="module")
def module_scopped_ahnlich_ai():
    port = 9001
    command = f"cargo run --bin ahnlich-ai run --supported-models all-minilm-l6-v2,resnet-50 --port {port}".split(
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
