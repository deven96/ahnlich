import os
import pathlib
import sys

from grpc_tools import protoc

PROTO_DIR = "../../protos/"
OUT_DIR = "./ahnlich_client_py/grpc/"

# Ensure the output directory exists
pathlib.Path(OUT_DIR).mkdir(parents=True, exist_ok=True)


def generate_from_protos():
    proto_files = [str(p) for p in pathlib.Path(PROTO_DIR).rglob("*.proto")]
    if not proto_files:
        print("No .proto files found.")
        return

    command = [
        "grpc_tools.protoc",
        f"--proto_path={PROTO_DIR}",
        f"--python_betterproto_out={OUT_DIR}",
    ] + proto_files

    if protoc.main(command) != 0:
        raise RuntimeError("Protobuf compilation failed")


if __name__ == "__main__":
    generate_from_protos()
