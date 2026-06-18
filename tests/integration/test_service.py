"""Integration tests for Service module."""

import oxitest
from oxi_nixinfra import Host, is_nixos
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
    assert status == "linked", (
        "NixOS services are 'linked' not 'enabled'"
        " — check systemctl is-enabled output parsing"
    )


@oxitest.mark.nixos
def test_nix_daemon_store_path(host: Fixture[Host]):
    path = host.service("nix-daemon").store_path()
    assert path is not None, (
        "NixOS-managed service should resolve to a /nix/store/ path"
        " — check systemctl show FragmentPath parsing"
    )
    assert path.startswith("/nix/store/"), (
        "store_path() returned a path outside /nix/store/"
        " — FragmentPath resolved to an unexpected target"
    )


@oxitest.mark.nixos
def test_nix_daemon_exists(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.exists(), "nix-daemon unit file not found in systemctl list-unit-files"


def test_nonexistent_service(host: Fixture[Host]):
    svc = host.service("nonexistent-service-12345")
    assert not svc.exists(), "fabricated service name should not match any unit file"


@oxitest.mark.skip(
    when=is_nixos(),
    reason="NixOS symlinks system-units to /nix/store/",
)
def test_nonexistent_service_not_managed(host: Fixture[Host]):
    svc = host.service("nonexistent-service-12345")
    assert not svc.is_managed(), "fabricated service should not be NixOS-managed"


@oxitest.mark.nixos
def test_service_properties(host: Fixture[Host]):
    props = host.service("nix-daemon").properties()
    assert isinstance(props, dict), (
        "properties() should parse 'systemctl show' key=value lines into a dict"
    )
    assert "Type" in props, (
        "'Type' is a standard systemd property"
        " — systemctl show output parsing may be broken"
    )
