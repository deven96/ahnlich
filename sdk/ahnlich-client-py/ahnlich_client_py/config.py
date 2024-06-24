import os
from pathlib import Path


class Config:

    def __init__(self) -> None:

        self.HEADER = b"AHNLICH;"
        self.BUFFER_SIZE = 1024
        self.PACKAGE_NAME = "ahnlich-client-py"
        self.BASE_DIR = Path(__file__).resolve().parent.parent
        self.AHNLICH_BIN_DIR = self.BASE_DIR.parent.parent / "ahnlich"
        # connection pool
        self.POOL_IDLE_TIMEOUT = float(os.environ.get("AHNLICH_IDLE_TIMEOUT", 30.0))
        self.POOL_MAX_LIFETIME = float(
            os.environ.get("AHNLICH_POOL_MAX_LIFETIME", 600.0)
        )

        self.POOL_MIN_IDLE_CONNECTIONS = int(
            os.environ.get("AHNLICH_POOL_MIN_IDLE_CONNECTION", 3)
        )
        self.POOL_MAX_SIZE = int(os.environ.get("AHNLICH_MAX_POOL_SIZE", 10))

        # background_collector=True,
        self.POOL_ENABLE_BACKGROUND_COLLECTOR = bool(
            int(os.environ.get("AHNLICH_POOL_ENABLE_BACKGROUND_COLLECTOR", 1))
        )
        self.POOL_DISPOSE_BATCH_SIZE = int(
            os.environ.get("AHNLICH_POOL_DISPOSE_BATCH_SIZE", 0)
        )


service_config = Config()
