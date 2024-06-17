import pytest
from internals import protocol
import os




@pytest.fixture
def base_protocol():
    host = os.environ.get("AHNLICH_DB_HOST", "127.0.0.1")
    port = int(os.environ.get("AHNLICH_DB_PORT", 1369))
    return protocol.AhnlichProtocol(address=host, port=port)
