"""Shared fixtures for integration tests."""

from oxi_nixinfra import Host
from oxitest import Fixtures

fixtures = Fixtures()


@fixtures.fixture
def host() -> Host:
    """Local host fixture for integration testing."""
    return Host._from_config("local://")
