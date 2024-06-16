import pytest
from internals import protocol


@pytest.fixture
def base_protocol():
    return protocol.AhnlichProtocol(address="127.0.0.1", port=1369)
