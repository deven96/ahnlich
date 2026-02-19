import asyncio
import hashlib
import os
import random
import signal
import socket
import ssl
import subprocess
import tempfile
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


# --- Auth helpers ---


def hash_api_key(api_key: str) -> str:
    return hashlib.sha256(api_key.encode()).hexdigest()


def write_auth_config(tmpdir: str, users: dict) -> str:
    path = os.path.join(tmpdir, "auth.toml")
    lines = ["[users]"]
    for username, api_key in users.items():
        lines.append(f'{username} = "{hash_api_key(api_key)}"')
    lines += ["", "[security]", "min_key_length = 8", ""]
    with open(path, "w") as f:
        f.write("\n".join(lines))
    return path


def generate_self_signed_cert(tmpdir: str) -> tuple:
    cert_path = os.path.join(tmpdir, "server.crt")
    key_path = os.path.join(tmpdir, "server.key")
    subprocess.run(
        [
            "openssl",
            "req",
            "-x509",
            "-newkey",
            "ec",
            "-pkeyopt",
            "ec_paramgen_curve:P-256",
            "-keyout",
            key_path,
            "-out",
            cert_path,
            "-days",
            "1",
            "-nodes",
            "-subj",
            "/CN=localhost",
            "-addext",
            "subjectAltName=DNS:localhost,IP:127.0.0.1",
        ],
        check=True,
        capture_output=True,
    )
    return cert_path, key_path


def make_client_ssl_context(cert_path: str) -> ssl.SSLContext:
    ctx = ssl.create_default_context()
    ctx.load_verify_locations(cafile=cert_path)
    ctx.check_hostname = False
    return ctx


def auth_metadata(username: str, api_key: str) -> list:
    return [("authorization", f"Bearer {username}:{api_key}")]


@pytest.fixture
def ahnlich_auth_setup():
    tmpdir = tempfile.mkdtemp()
    cert_path, key_path = generate_self_signed_cert(tmpdir)
    yield tmpdir, cert_path, key_path
    import shutil

    shutil.rmtree(tmpdir, ignore_errors=True)


@pytest.fixture
def spin_up_ahnlich_db_with_auth(ahnlich_auth_setup, db_random_port):
    tmpdir, cert_path, key_path = ahnlich_auth_setup
    port = db_random_port
    auth_config = write_auth_config(tmpdir, {"alice": "alicepass", "bob": "bobspass1"})
    command = (
        f"cargo run --bin ahnlich-db run --port {port} "
        f"--enable-auth --auth-config {auth_config} "
        f"--tls-cert {cert_path} --tls-key {key_path}"
    ).split()
    process = subprocess.Popen(args=command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(port):
        time.sleep(0.2)
    yield port, cert_path
    os.kill(process.pid, signal.SIGINT)
    process.wait(5)


@pytest.fixture
def spin_up_ahnlich_ai_with_auth(ahnlich_auth_setup, ai_random_port):
    tmpdir, cert_path, key_path = ahnlich_auth_setup
    db_port = random.randint(7000, 7999)
    db_command = f"cargo run --bin ahnlich-db run --port {db_port}".split()
    db_process = subprocess.Popen(args=db_command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(db_port):
        time.sleep(0.2)

    ai_port = ai_random_port
    ai_auth_config = write_auth_config(tmpdir, {"aiuser": "aipassword"})
    ai_command = (
        f"cargo run --bin ahnlich-ai run --port {ai_port} "
        f"--supported-models all-minilm-l6-v2,resnet-50 "
        f"--db-host 127.0.0.1 --db-port {db_port} "
        f"--enable-auth --auth-config {ai_auth_config} "
        f"--tls-cert {cert_path} --tls-key {key_path}"
    ).split()
    ai_process = subprocess.Popen(args=ai_command, cwd=config.AHNLICH_BIN_DIR)
    while not is_port_occupied(ai_port):
        time.sleep(0.2)
    time.sleep(1)

    yield ai_port, cert_path

    os.kill(ai_process.pid, signal.SIGINT)
    ai_process.wait(5)
    os.kill(db_process.pid, signal.SIGINT)
    db_process.wait(5)
