"""Integration tests for SystemInfo module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_type_is_linux(host: Fixture[Host]):
    sys_type = host.system_info().type()
    assert sys_type == "linux", f"uname -s returned {sys_type!r}, expected 'linux'"


@oxitest.mark.nixos
def test_distribution_is_nixos(host: Fixture[Host]):
    distro = host.system_info().distribution()
    assert distro == "nixos", f"/etc/os-release ID={distro!r}, expected 'nixos'"


def test_arch_not_empty(host: Fixture[Host]):
    arch = host.system_info().arch()
    assert isinstance(arch, str), f"arch() returned {type(arch).__name__}, expected str"
    assert len(arch) > 0, "'uname -m' returned an empty string"


def test_release_not_empty(host: Fixture[Host]):
    release = host.system_info().release()
    assert isinstance(release, str), (
        f"release() returned {type(release).__name__}, expected str"
    )
    assert len(release) > 0, "/etc/os-release VERSION_ID is empty"
