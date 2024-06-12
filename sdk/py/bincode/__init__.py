# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

import dataclasses
import collections
import io
import struct
import typing
from copy import copy
from typing import get_type_hints

import serde_types as st
import serde_binary as sb

# Maximum length in practice for sequences (e.g. in Java).
MAX_LENGTH = (1 << 31) - 1


class BincodeSerializer(sb.BinarySerializer):
    def __init__(self):
        super().__init__(output=io.BytesIO(), container_depth_budget=None)

    def serialize_f32(self, value: st.float32):
        self.output.write(struct.pack("<f", value))

    def serialize_f64(self, value: st.float64):
        self.output.write(struct.pack("<d", value))

    def serialize_len(self, value: int):
        if value > MAX_LENGTH:
            raise st.SerializationError("Length exceeds the maximum supported value.")
        self.output.write(int(value).to_bytes(8, "little", signed=False))

    def serialize_variant_index(self, value: int):
        self.output.write(int(value).to_bytes(4, "little", signed=False))

    def sort_map_entries(self, offsets: typing.List[int]):
        pass


class BincodeDeserializer(sb.BinaryDeserializer):
    def __init__(self, content):
        super().__init__(input=io.BytesIO(content), container_depth_budget=None)

    def deserialize_f32(self) -> st.float32:
        (value,) = struct.unpack("<f", self.read(4))
        return st.float32(value)

    def deserialize_f64(self) -> st.float64:
        (value,) = struct.unpack("<d", self.read(8))
        return st.float64(value)

    def deserialize_len(self) -> int:
        value = int.from_bytes(self.read(8), byteorder="little", signed=False)
        if value > MAX_LENGTH:
            raise st.DeserializationError("Length exceeds the maximum supported value.")
        return value

    def deserialize_variant_index(self) -> int:
        return int.from_bytes(self.read(4), byteorder="little", signed=False)

    def check_that_key_slices_are_increasing(
        self, slice1: typing.Tuple[int, int], slice2: typing.Tuple[int, int]
    ):
        pass


def serialize(obj: typing.Any, obj_type) -> bytes:
    serializer = BincodeSerializer()
    serializer.serialize_any(obj, obj_type)
    return serializer.get_buffer()


def deserialize(content: bytes, obj_type) -> typing.Tuple[typing.Any, bytes]:
    deserializer = BincodeDeserializer(content)
    value = deserializer.deserialize_any(obj_type)
    return value, deserializer.get_remaining_buffer()
