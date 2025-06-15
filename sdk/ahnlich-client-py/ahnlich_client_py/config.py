from dataclasses import dataclass
from pathlib import Path

TRACE_HEADER: str = "ahnlich-trace-id"
PACKAGE_NAME = "ahnlich-client-py"
BASE_DIR = Path(__file__).resolve().parent.parent
AHNLICH_BIN_DIR = BASE_DIR.parent.parent / "ahnlich"
