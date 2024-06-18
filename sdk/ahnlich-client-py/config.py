from pathlib import Path

from numpy import who

HEADER = b"AHNLICH;"
BUFFER_SIZE = 1024
PACKAGE_NAME = "ahnlich-client-py"
BASE_DIR = Path(__file__).resolve().parent

AHNLISH_BIN_DIR = BASE_DIR.parent.parent / "ahnlich"
