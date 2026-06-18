"""Integration tests for Sysctl module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_sysctl_kernel_hostname(host: Fixture[Host]):
    val = host.sysctl("kernel.hostname").value()
    assert isinstance(val, str), "sysctl value should be a string"
    assert len(val) > 0, "kernel.hostname should not be empty"


def test_sysctl_exists_true(host: Fixture[Host]):
    assert host.sysctl("kernel.hostname").exists(), (
        "kernel.hostname is a standard sysctl key — should exist on all Linux systems"
    )


def test_sysctl_exists_false(host: Fixture[Host]):
    assert not host.sysctl("nonexistent.fake.key.12345").exists(), (
        "fabricated sysctl key should not exist"
    )
