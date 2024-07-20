import typing

import numpy as np

from ahnlich_client_py import exceptions as ah_exceptions
from ahnlich_client_py.internals import ai_query
from ahnlich_client_py.internals import serde_types as st
from ahnlich_client_py.libs import NonZeroSizeInteger


class AhnlichAIRequestBuilder:
    def __init__(self) -> None:
        self.queries: typing.List[ai_query.AIQuery] = []
