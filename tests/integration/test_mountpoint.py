"""Integration tests for MountPoint module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_root_mount_exists(host: Fixture[Host]):
    assert host.mountpoint("/").exists(), "root filesystem should always be mounted"


def test_root_filesystem_type(host: Fixture[Host]):
    fs = host.mountpoint("/").filesystem()
    expected = host.run("findmnt", "-n", "-o", "FSTYPE", "/").stdout.strip()
    assert fs == expected, (
        f"mountpoint('/').filesystem() returned '{fs}'"
        f" but 'findmnt -n -o FSTYPE /' returned '{expected}'"
        " — check json_field('fstype') parsing"
    )


def test_root_device(host: Fixture[Host]):
    dev = host.mountpoint("/").device()
    expected = host.run("findmnt", "-n", "-o", "SOURCE", "/").stdout.strip()
    assert dev == expected, (
        f"mountpoint('/').device() returned '{dev}'"
        f" but 'findmnt -n -o SOURCE /' returned '{expected}'"
        " — check json_field('source') parsing"
    )


def test_root_options(host: Fixture[Host]):
    opts = host.mountpoint("/").options()
    assert isinstance(opts, list), "options() should return a list"
    assert len(opts) > 0, "root mount should have at least one option"


def test_nonexistent_mount(host: Fixture[Host]):
    assert not host.mountpoint("/nonexistent/path/12345").exists(), (
        "fabricated path should not be a mount point"
    )
