from dataclasses import dataclass
from pathlib import Path

HEADER = b"AHNLICH;"
BUFFER_SIZE = 1024
PACKAGE_NAME = "ahnlich-client-py"
BASE_DIR = Path(__file__).resolve().parent.parent
AHNLICH_BIN_DIR = BASE_DIR.parent.parent / "ahnlich"


@dataclass
class AhnlichPoolSettings:
    idle_timeout: float = 30.0
    max_lifetime: float = 600.0
    min_idle_connections: int = 3
    max_pool_size: int = 10
    enable_background_collector: bool = True
    dispose_batch_size: int = 0
