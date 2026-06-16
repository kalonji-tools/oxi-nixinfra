"""Integration tests for File module."""
from oxi_nixinfra import Host
from oxitest import Fixture

def test_etc_os_release_exists(host: Fixture[Host]):
    assert host.file("/etc/os-release").exists()

def test_etc_os_release_is_file(host: Fixture[Host]):
    assert host.file("/etc/os-release").is_file()

def test_etc_is_directory(host: Fixture[Host]):
    assert host.file("/etc").is_directory()

def test_nonexistent_file(host: Fixture[Host]):
    assert not host.file("/nonexistent-path-12345").exists()

def test_file_content(host: Fixture[Host]):
    content = host.file("/etc/os-release").content()
    assert "nixos" in content.lower()

def test_file_user_root(host: Fixture[Host]):
    assert host.file("/etc/os-release").user() == "root"

def test_file_mode(host: Fixture[Host]):
    mode = host.file("/etc/os-release").mode()
    assert isinstance(mode, int)
    assert mode > 0
