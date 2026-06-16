"""Integration tests for Service module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


@oxitest.mark.nixos
def test_nix_daemon_running(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.is_running(), (
        "nix-daemon is not running — is the Nix daemon service active?"
    )


@oxitest.mark.nixos
def test_accounts_daemon_enabled(host: Fixture[Host]):
    svc = host.service("accounts-daemon")
    assert svc.is_enabled(), (
        "accounts-daemon is not enabled — check systemctl is-enabled accounts-daemon"
    )


@oxitest.mark.nixos
def test_nix_daemon_exists(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.exists(), "nix-daemon unit file not found in systemctl list-unit-files"


def test_nonexistent_service(host: Fixture[Host]):
    svc = host.service("nonexistent-service-12345")
    assert not svc.exists(), "fabricated service name should not match any unit file"


@oxitest.mark.nixos
def test_service_properties(host: Fixture[Host]):
    props = host.service("nix-daemon").properties()
    assert isinstance(props, dict), (
        f"expected dict from systemctl show, got {type(props)}"
    )
    assert "Type" in props, (
        f"'Type' missing from systemctl show, keys: {sorted(props)[:10]}"
    )
