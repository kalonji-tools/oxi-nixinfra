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
def test_nix_daemon_is_managed(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.is_managed(), (
        "nix-daemon should be NixOS-managed (symlink to /nix/store/)"
    )


@oxitest.mark.nixos
def test_nix_daemon_enablement_status(host: Fixture[Host]):
    status = host.service("nix-daemon").enablement_status()
    assert status == "linked", f"nix-daemon should be 'linked' on NixOS, got {status!r}"


@oxitest.mark.nixos
def test_nix_daemon_store_path(host: Fixture[Host]):
    path = host.service("nix-daemon").store_path()
    assert path is not None, "nix-daemon store_path should not be None"
    assert path.startswith("/nix/store/"), (
        f"nix-daemon store_path should start with /nix/store/, got {path!r}"
    )


@oxitest.mark.nixos
def test_nix_daemon_exists(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.exists(), "nix-daemon unit file not found in systemctl list-unit-files"


def test_nonexistent_service(host: Fixture[Host]):
    svc = host.service("nonexistent-service-12345")
    assert not svc.exists(), "fabricated service name should not match any unit file"


def test_nonexistent_service_not_managed(host: Fixture[Host]):
    svc = host.service("nonexistent-service-12345")
    assert not svc.is_managed(), "fabricated service should not be NixOS-managed"


@oxitest.mark.nixos
def test_service_properties(host: Fixture[Host]):
    props = host.service("nix-daemon").properties()
    assert isinstance(props, dict), (
        f"expected dict from systemctl show, got {type(props)}"
    )
    assert "Type" in props, (
        f"'Type' missing from systemctl show, keys: {sorted(props)[:10]}"
    )
