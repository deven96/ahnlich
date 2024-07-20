import typing

from ahnlich_client_py import builders
from ahnlich_client_py.config import AhnlichDBPoolSettings
from ahnlich_client_py.internals import db_query, db_response
from ahnlich_client_py.internals import serde_types as st
from ahnlich_client_py.internals.base_client import BaseClient


class AhnlichAIClient(BaseClient):
    """Wrapper for interacting with Ahnlich database"""
