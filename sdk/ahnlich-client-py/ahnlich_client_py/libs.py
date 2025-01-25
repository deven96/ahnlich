import typing

import numpy as np

from ahnlich_client_py import exceptions as ah_exceptions
from ahnlich_client_py.internals import serde_types as st


class NonZeroSizeInteger:
    def __init__(self, num: st.uint64) -> None:
        if num <= 0:
            raise ah_exceptions.AhnlichValidationError(
                "Ahnlich expects a Non zero value as integers"
            )
        self.value = num


def create_store_key(
    data: typing.List[float], v: int = 1
) -> typing.Sequence[st.float32]:
    return [st.float32(f) for f in data]
