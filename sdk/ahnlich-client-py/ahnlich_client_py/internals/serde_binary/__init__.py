# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

"""
Module describing the "binary" serialization formats.

Note: This internal module is currently only meant to share code between the BCS and bincode formats. Internal APIs could change in the future.
"""

import collections
import dataclasses
import io
import typing
from typing import get_type_hints

from ahnlich_client_py.internals import serde_types as st


@dataclasses.dataclass
class BinarySerializer:
    """Serialization primitives for binary formats (abstract class).

    "Binary" serialization formats may differ in the way they encode sequence lengths, variant
    index, and how they sort map entries (or not).
    """

    output: io.BytesIO
    container_depth_budget: typing.Optional[int]
    primitive_type_serializer: typing.Mapping = dataclasses.field(init=False)

    def __post_init__(self):
        self.primitive_type_serializer = {
            bool: self.serialize_bool,
            st.uint8: self.serialize_u8,
            st.uint16: self.serialize_u16,
            st.uint32: self.serialize_u32,
            st.uint64: self.serialize_u64,
            st.uint128: self.serialize_u128,
            st.int8: self.serialize_i8,
            st.int16: self.serialize_i16,
            st.int32: self.serialize_i32,
            st.int64: self.serialize_i64,
            st.int128: self.serialize_i128,
            st.float32: self.serialize_f32,
            st.float64: self.serialize_f64,
            st.unit: self.serialize_unit,
            st.char: self.serialize_char,
            str: self.serialize_str,
            bytes: self.serialize_bytes,
        }

    def serialize_bytes(self, value: bytes):
        self.serialize_len(len(value))
        self.output.write(value)

    def serialize_str(self, value: str):
        self.serialize_bytes(value.encode())

    def serialize_unit(self, value: st.unit):
        pass

    def serialize_bool(self, value: bool):
        self.output.write(int(value).to_bytes(1, "little", signed=False))

    def serialize_u8(self, value: st.uint8):
        self.output.write(int(value).to_bytes(1, "little", signed=False))

    def serialize_u16(self, value: st.uint16):
        self.output.write(int(value).to_bytes(2, "little", signed=False))

    def serialize_u32(self, value: st.uint32):
        self.output.write(int(value).to_bytes(4, "little", signed=False))

    def serialize_u64(self, value: st.uint64):
        self.output.write(int(value).to_bytes(8, "little", signed=False))

    def serialize_u128(self, value: st.uint128):
        self.output.write(int(value).to_bytes(16, "little", signed=False))

    def serialize_i8(self, value: st.uint8):
        self.output.write(int(value).to_bytes(1, "little", signed=True))

    def serialize_i16(self, value: st.uint16):
        self.output.write(int(value).to_bytes(2, "little", signed=True))

    def serialize_i32(self, value: st.uint32):
        self.output.write(int(value).to_bytes(4, "little", signed=True))

    def serialize_i64(self, value: st.uint64):
        self.output.write(int(value).to_bytes(8, "little", signed=True))

    def serialize_i128(self, value: st.uint128):
        self.output.write(int(value).to_bytes(16, "little", signed=True))

    def serialize_f32(self, value: st.float32):
        raise NotImplementedError

    def serialize_f64(self, value: st.float64):
        raise NotImplementedError

    def serialize_char(self, value: st.char):
        raise NotImplementedError

    def get_buffer_offset(self) -> int:
        return len(self.output.getbuffer())

    def get_buffer(self) -> bytes:
        return self.output.getvalue()

    def increase_container_depth(self):
        if self.container_depth_budget is not None:
            if self.container_depth_budget == 0:
                raise st.SerializationError("Exceeded maximum container depth")
            self.container_depth_budget -= 1

    def decrease_container_depth(self):
        if self.container_depth_budget is not None:
            self.container_depth_budget += 1

    def serialize_len(self, value: int):
        raise NotImplementedError

    def serialize_variant_index(self, value: int):
        raise NotImplementedError

    def sort_map_entries(self, offsets: typing.List[int]):
        raise NotImplementedError

    # noqa: C901
    def serialize_any(self, obj: typing.Any, obj_type):
        if obj_type in self.primitive_type_serializer:
            self.primitive_type_serializer[obj_type](obj)

        elif hasattr(obj_type, "__origin__"):  # Generic type
            types = getattr(obj_type, "__args__")

            if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
                assert len(types) == 1
                item_type = types[0]
                self.serialize_len(len(obj))
                for item in obj:
                    self.serialize_any(item, item_type)

            elif getattr(obj_type, "__origin__") == tuple:  # Tuple
                if len(types) != 1 or types[0] != ():
                    for i in range(len(obj)):
                        self.serialize_any(obj[i], types[i])

            elif getattr(obj_type, "__origin__") == typing.Union:  # Option
                assert len(types) == 2 and types[1] == type(None)
                if obj is None:
                    self.output.write(b"\x00")
                else:
                    self.output.write(b"\x01")
                    self.serialize_any(obj, types[0])

            elif getattr(obj_type, "__origin__") == dict:  # Map
                assert len(types) == 2
                self.serialize_len(len(obj))
                offsets = []
                for key, value in obj.items():
                    offsets.append(self.get_buffer_offset())
                    self.serialize_any(key, types[0])
                    self.serialize_any(value, types[1])
                self.sort_map_entries(offsets)

            else:
                raise st.SerializationError("Unexpected type", obj_type)

        else:
            if not dataclasses.is_dataclass(obj_type):  # Enum
                if not hasattr(obj_type, "VARIANTS"):
                    raise st.SerializationError("Unexpected type", obj_type)
                if not hasattr(obj, "INDEX"):
                    raise st.SerializationError(
                        "Wrong Value for the type", obj, obj_type
                    )
                self.serialize_variant_index(obj.__class__.INDEX)
                # Proceed to variant
                obj_type = obj_type.VARIANTS[obj.__class__.INDEX]
                if not dataclasses.is_dataclass(obj_type):
                    raise st.SerializationError("Unexpected type", obj_type)

            # pyre-ignore
            if not isinstance(obj, obj_type):
                raise st.SerializationError("Wrong Value for the type", obj, obj_type)

            # Content of struct or variant
            fields = dataclasses.fields(obj_type)
            types = get_type_hints(obj_type)
            self.increase_container_depth()
            for field in fields:
                field_value = obj.__dict__[field.name]
                field_type = types[field.name]
                self.serialize_any(field_value, field_type)
            self.decrease_container_depth()


@dataclasses.dataclass
class BinaryDeserializer:
    """Deserialization primitives for binary formats (abstract class).

    "Binary" serialization formats may differ in the way they encode sequence lengths, variant
    index, and how they verify the ordering of keys in map entries (or not).
    """

    input: io.BytesIO
    container_depth_budget: typing.Optional[int]
    primitive_type_deserializer: typing.Mapping = dataclasses.field(init=False)

    def __post_init__(self):
        self.primitive_type_deserializer = {
            bool: self.deserialize_bool,
            st.uint8: self.deserialize_u8,
            st.uint16: self.deserialize_u16,
            st.uint32: self.deserialize_u32,
            st.uint64: self.deserialize_u64,
            st.uint128: self.deserialize_u128,
            st.int8: self.deserialize_i8,
            st.int16: self.deserialize_i16,
            st.int32: self.deserialize_i32,
            st.int64: self.deserialize_i64,
            st.int128: self.deserialize_i128,
            st.float32: self.deserialize_f32,
            st.float64: self.deserialize_f64,
            st.unit: self.deserialize_unit,
            st.char: self.deserialize_char,
            str: self.deserialize_str,
            bytes: self.deserialize_bytes,
        }

    def read(self, length: int) -> bytes:
        value = self.input.read(length)
        if value is None or len(value) < length:
            raise st.DeserializationError("Input is too short")
        return value

    def deserialize_bytes(self) -> bytes:
        length = self.deserialize_len()
        return self.read(length)

    def deserialize_str(self) -> str:
        content = self.deserialize_bytes()
        try:
            return content.decode()
        except UnicodeDecodeError:
            raise st.DeserializationError("Invalid unicode string:", content)

    def deserialize_unit(self) -> st.unit:
        pass

    def deserialize_bool(self) -> bool:
        b = int.from_bytes(self.read(1), byteorder="little", signed=False)
        if b == 0:
            return False
        elif b == 1:
            return True
        else:
            raise st.DeserializationError("Unexpected boolean value:", b)

    def deserialize_u8(self) -> st.uint8:
        return st.uint8(int.from_bytes(self.read(1), byteorder="little", signed=False))

    def deserialize_u16(self) -> st.uint16:
        return st.uint16(int.from_bytes(self.read(2), byteorder="little", signed=False))

    def deserialize_u32(self) -> st.uint32:
        return st.uint32(int.from_bytes(self.read(4), byteorder="little", signed=False))

    def deserialize_u64(self) -> st.uint64:
        return st.uint64(int.from_bytes(self.read(8), byteorder="little", signed=False))

    def deserialize_u128(self) -> st.uint128:
        return st.uint128(
            int.from_bytes(self.read(16), byteorder="little", signed=False)
        )

    def deserialize_i8(self) -> st.int8:
        return st.int8(int.from_bytes(self.read(1), byteorder="little", signed=True))

    def deserialize_i16(self) -> st.int16:
        return st.int16(int.from_bytes(self.read(2), byteorder="little", signed=True))

    def deserialize_i32(self) -> st.int32:
        return st.int32(int.from_bytes(self.read(4), byteorder="little", signed=True))

    def deserialize_i64(self) -> st.int64:
        return st.int64(int.from_bytes(self.read(8), byteorder="little", signed=True))

    def deserialize_i128(self) -> st.int128:
        return st.int128(int.from_bytes(self.read(16), byteorder="little", signed=True))

    def deserialize_f32(self) -> st.float32:
        raise NotImplementedError

    def deserialize_f64(self) -> st.float64:
        raise NotImplementedError

    def deserialize_char(self) -> st.char:
        raise NotImplementedError

    def get_buffer_offset(self) -> int:
        return self.input.tell()

    def get_remaining_buffer(self) -> bytes:
        buf = self.input.getbuffer()
        offset = self.get_buffer_offset()
        return bytes(buf[offset:])

    def increase_container_depth(self):
        if self.container_depth_budget is not None:
            if self.container_depth_budget == 0:
                raise st.DeserializationError("Exceeded maximum container depth")
            self.container_depth_budget -= 1

    def decrease_container_depth(self):
        if self.container_depth_budget is not None:
            self.container_depth_budget += 1

    def deserialize_len(self) -> int:
        raise NotImplementedError

    def deserialize_variant_index(self) -> int:
        raise NotImplementedError

    def check_that_key_slices_are_increasing(
        self, slice1: typing.Tuple[int, int], slice2: typing.Tuple[int, int]
    ) -> bool:
        raise NotImplementedError

    # noqa
    def deserialize_any(self, obj_type) -> typing.Any:
        if obj_type in self.primitive_type_deserializer:
            return self.primitive_type_deserializer[obj_type]()

        elif hasattr(obj_type, "__origin__"):  # Generic type
            types = getattr(obj_type, "__args__")
            if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
                assert len(types) == 1
                item_type = types[0]
                length = self.deserialize_len()
                result = []
                for i in range(0, length):
                    item = self.deserialize_any(item_type)
                    result.append(item)

                return result

            elif getattr(obj_type, "__origin__") == tuple:  # Tuple
                result = []
                if len(types) == 1 and types[0] == ():
                    return tuple()
                for i in range(len(types)):
                    item = self.deserialize_any(types[i])
                    result.append(item)
                return tuple(result)

            elif getattr(obj_type, "__origin__") == typing.Union:  # Option
                assert len(types) == 2 and types[1] == type(None)
                tag = int.from_bytes(self.read(1), byteorder="little", signed=False)
                if tag == 0:
                    return None
                elif tag == 1:
                    return self.deserialize_any(types[0])
                else:
                    raise st.DeserializationError("Wrong tag for Option value")

            elif getattr(obj_type, "__origin__") == dict:  # Map
                assert len(types) == 2
                length = self.deserialize_len()
                result = dict()
                previous_key_slice = None
                for i in range(0, length):
                    key_start = self.get_buffer_offset()
                    key = self.deserialize_any(types[0])
                    key_end = self.get_buffer_offset()
                    value = self.deserialize_any(types[1])

                    key_slice = (key_start, key_end)
                    if previous_key_slice is not None:
                        self.check_that_key_slices_are_increasing(
                            previous_key_slice, key_slice
                        )
                    previous_key_slice = key_slice

                    result[key] = value

                return result

            else:
                raise st.DeserializationError("Unexpected type", obj_type)

        else:
            # handle structs
            if dataclasses.is_dataclass(obj_type):
                values = []
                fields = dataclasses.fields(obj_type)
                typing_hints = get_type_hints(obj_type)
                self.increase_container_depth()
                for field in fields:
                    field_type = typing_hints[field.name]
                    field_value = self.deserialize_any(field_type)
                    values.append(field_value)
                self.decrease_container_depth()
                return obj_type(*values)

            # handle variant
            elif hasattr(obj_type, "VARIANTS"):
                variant_index = self.deserialize_variant_index()
                if variant_index not in range(len(obj_type.VARIANTS)):
                    raise st.DeserializationError(
                        "Unexpected variant index", variant_index
                    )
                new_type = obj_type.VARIANTS[variant_index]
                return self.deserialize_any(new_type)

            else:
                raise st.DeserializationError("Unexpected type", obj_type)
