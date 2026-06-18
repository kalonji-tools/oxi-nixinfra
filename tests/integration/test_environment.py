"""Integration tests for Environment module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_env_get_path(host: Fixture[Host]):
    val = host.environment().get("PATH")
    assert val is not None, "PATH should be set in any Unix environment"
    assert "/" in val, "PATH should contain at least one directory separator"


def test_env_get_nonexistent(host: Fixture[Host]):
    val = host.environment().get("NONEXISTENT_VAR_12345")
    assert val is None, "fabricated env var should return None"


def test_env_exists_true(host: Fixture[Host]):
    assert host.environment().exists("PATH"), (
        "PATH is a standard environment variable — should exist"
    )


def test_env_exists_false(host: Fixture[Host]):
    assert not host.environment().exists("NONEXISTENT_VAR_12345"), (
        "fabricated env var should not exist"
    )
