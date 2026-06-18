"""Integration tests for MountPoint module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_root_mount_exists(host: Fixture[Host]):
    assert host.mountpoint("/").exists(), "root filesystem should always be mounted"


def test_root_filesystem_type(host: Fixture[Host]):
    fs = host.mountpoint("/").filesystem()
    assert isinstance(fs, str), "filesystem() should return a string"
    assert len(fs) > 0, "filesystem type should not be empty"


def test_root_device(host: Fixture[Host]):
    dev = host.mountpoint("/").device()
    assert isinstance(dev, str), "device() should return a string"
    assert len(dev) > 0, "device should not be empty"


def test_root_options(host: Fixture[Host]):
    opts = host.mountpoint("/").options()
    assert isinstance(opts, list), "options() should return a list"
    assert len(opts) > 0, "root mount should have at least one option"


def test_nonexistent_mount(host: Fixture[Host]):
    assert not host.mountpoint("/nonexistent/path/12345").exists(), (
        "fabricated path should not be a mount point"
    )
