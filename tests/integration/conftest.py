"""Shared fixtures for integration tests."""

import os

from oxi_nixinfra import Host
from oxitest import Fixtures

fixtures = Fixtures()


@fixtures.fixture
def host() -> Host:
    """Host fixture -- respects OXITEST_HOST env var."""
    return Host._from_config(os.environ.get("OXITEST_HOST", "local://"))
