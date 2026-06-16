"""Integration tests for Service module."""
import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


@oxitest.mark.nixos
def test_nix_daemon_running(host: Fixture[Host]):
    assert host.service("nix-daemon").is_running()


@oxitest.mark.nixos
def test_accounts_daemon_enabled(host: Fixture[Host]):
    assert host.service("accounts-daemon").is_enabled()


@oxitest.mark.nixos
def test_nix_daemon_exists(host: Fixture[Host]):
    assert host.service("nix-daemon").exists()


def test_nonexistent_service(host: Fixture[Host]):
    assert not host.service("nonexistent-service-12345").exists()


@oxitest.mark.nixos
def test_service_properties(host: Fixture[Host]):
    props = host.service("nix-daemon").properties()
    assert isinstance(props, dict)
    assert "Type" in props
