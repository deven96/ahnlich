# Generated by the protocol buffer compiler.  DO NOT EDIT!
# sources: predicate.proto
# plugin: python-betterproto
# This file has been @generated

from dataclasses import dataclass
from typing import List

import betterproto

from .. import metadata as _metadata__


@dataclass(eq=False, repr=False)
class Predicate(betterproto.Message):
    equals: "Equals" = betterproto.message_field(1, group="kind")
    not_equals: "NotEquals" = betterproto.message_field(2, group="kind")
    in_: "In" = betterproto.message_field(3, group="kind")
    not_in: "NotIn" = betterproto.message_field(4, group="kind")


@dataclass(eq=False, repr=False)
class Equals(betterproto.Message):
    key: str = betterproto.string_field(1)
    value: "_metadata__.MetadataValue" = betterproto.message_field(2)


@dataclass(eq=False, repr=False)
class NotEquals(betterproto.Message):
    key: str = betterproto.string_field(1)
    value: "_metadata__.MetadataValue" = betterproto.message_field(2)


@dataclass(eq=False, repr=False)
class In(betterproto.Message):
    key: str = betterproto.string_field(1)
    values: List["_metadata__.MetadataValue"] = betterproto.message_field(2)


@dataclass(eq=False, repr=False)
class NotIn(betterproto.Message):
    key: str = betterproto.string_field(1)
    values: List["_metadata__.MetadataValue"] = betterproto.message_field(2)


@dataclass(eq=False, repr=False)
class PredicateCondition(betterproto.Message):
    value: "Predicate" = betterproto.message_field(1, group="kind")
    and_: "AndCondition" = betterproto.message_field(2, group="kind")
    or_: "OrCondition" = betterproto.message_field(3, group="kind")


@dataclass(eq=False, repr=False)
class AndCondition(betterproto.Message):
    left: "PredicateCondition" = betterproto.message_field(1)
    right: "PredicateCondition" = betterproto.message_field(2)


@dataclass(eq=False, repr=False)
class OrCondition(betterproto.Message):
    left: "PredicateCondition" = betterproto.message_field(1)
    right: "PredicateCondition" = betterproto.message_field(2)
