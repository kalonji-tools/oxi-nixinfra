"""Integration tests for Sysctl module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_sysctl_kernel_hostname(host: Fixture[Host]):
    val = host.sysctl("kernel.hostname").value()
    expected = host.run("hostname").stdout.strip()
    assert val == expected, (
        f"sysctl('kernel.hostname').value() returned '{val}'"
        f" but 'hostname' command returned '{expected}'"
        " — check sysctl -n output parsing"
    )


def test_sysctl_exists_true(host: Fixture[Host]):
    assert host.sysctl("kernel.hostname").exists(), (
        "kernel.hostname is a standard sysctl key — should exist on all Linux systems"
    )


def test_sysctl_exists_false(host: Fixture[Host]):
    assert not host.sysctl("nonexistent.fake.key.12345").exists(), (
        "fabricated sysctl key should not exist"
    )
