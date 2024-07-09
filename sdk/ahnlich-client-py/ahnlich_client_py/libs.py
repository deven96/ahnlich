import typing

import numpy as np

from ahnlich_client_py.internals import db_query
from ahnlich_client_py.internals import serde_types as st


def create_store_key(data: typing.List[float], v: int = 1) -> db_query.Array:
    np_array = np.array(data, dtype=np.float32)
    dimensions = (st.uint64(np_array.shape[0]),)
    store_key = db_query.Array(v=st.uint8(v), dim=dimensions, data=np_array.tolist())
    return store_key
