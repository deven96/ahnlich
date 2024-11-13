import typing

import numpy as np

from ahnlich_client_py import exceptions as ah_exceptions
from ahnlich_client_py.internals import db_query
from ahnlich_client_py.internals import serde_types as st


class NonZeroSizeInteger:
    def __init__(self, num: st.uint64) -> None:
        if num <= 0:
            raise ah_exceptions.AhnlichValidationError(
                "Ahnlich expects a Non zero value as integers"
            )
        self.value = num


def create_store_key(data: typing.List[float], v: int = 1) -> db_query.Array:
    np_array = np.array(data, dtype=np.float32)
    dimensions = (st.uint64(np_array.shape[0]),)
    store_key = db_query.Array(v=st.uint8(v), dim=dimensions, data=np_array.tolist())
    return store_key
