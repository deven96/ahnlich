import typing

import numpy as np

from ahnlich_client_py.internals import query
from ahnlich_client_py.internals import serde_types as st


def create_store_key(data: typing.List[float], v: int = 1) -> query.Array:
    np_array = np.array(data, dtype=np.float32)
    dimensions = (st.uint64(np_array.shape[0]),)
    store_key = query.Array(v=st.uint8(v), dim=dimensions, data=np_array.tolist())
    return store_key


class Singleton(type):
    instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls.instances:
            cls.instances[cls] = super(Singleton, cls).__call__(*args, **kwargs)
        return cls.instances[cls]

    @classmethod
    def reset(mcs):
        # for correct test
        mcs.instances = {}
