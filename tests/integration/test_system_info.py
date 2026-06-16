"""Integration tests for SystemInfo module."""
import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_type_is_linux(host: Fixture[Host]):
    assert host.system_info().type() == "linux"


@oxitest.mark.nixos
def test_distribution_is_nixos(host: Fixture[Host]):
    assert host.system_info().distribution() == "nixos"


def test_arch_not_empty(host: Fixture[Host]):
    arch = host.system_info().arch()
    assert isinstance(arch, str)
    assert len(arch) > 0


def test_release_not_empty(host: Fixture[Host]):
    release = host.system_info().release()
    assert isinstance(release, str)
    assert len(release) > 0
