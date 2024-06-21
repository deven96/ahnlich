from pathlib import Path

HEADER = b"AHNLICH;"
BUFFER_SIZE = 1024
PACKAGE_NAME = "ahnlich-client-py"
BASE_DIR = Path(__file__).resolve().parent.parent

AHNLICH_BIN_DIR = BASE_DIR.parent.parent / "ahnlich"
