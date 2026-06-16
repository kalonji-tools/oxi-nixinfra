"""Integration tests for User module."""
from oxi_nixinfra import Host
from oxitest import Fixture

def test_root_exists(host: Fixture[Host]):
    assert host.user("root").exists()

def test_root_uid(host: Fixture[Host]):
    assert host.user("root").uid() == 0

def test_root_home(host: Fixture[Host]):
    assert host.user("root").home() == "/root"

def test_nonexistent_user(host: Fixture[Host]):
    assert not host.user("nonexistent-user-12345").exists()

def test_current_user(host: Fixture[Host]):
    name = host.user().name()
    assert isinstance(name, str)
    assert len(name) > 0
