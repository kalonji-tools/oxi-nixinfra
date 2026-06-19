"""Integration tests for NixPackage module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


@oxitest.mark.nixos
def test_nix_is_installed(host: Fixture[Host]):
    assert host.nix_package("nix").is_installed(), (
        "nix package should be in /run/current-system/sw references"
    )


@oxitest.mark.nixos
def test_coreutils_version(host: Fixture[Host]):
    version = host.nix_package("coreutils-full").version()
    assert version is not None, (
        "coreutils-full should have a parseable version from its store path"
    )
    assert "." in version, (
        "version should be semver-like (e.g. 9.8)"
        " — check parse_version_from_store_path logic"
    )


@oxitest.mark.nixos
def test_nix_store_path(host: Fixture[Host]):
    path = host.nix_package("nix").store_path()
    assert path is not None and path.startswith("/nix/store/"), (
        "installed package should resolve to a /nix/store/ path"
    )


def test_nonexistent_package_not_installed(host: Fixture[Host]):
    assert not host.nix_package("nonexistent-pkg-12345").is_installed(), (
        "fabricated package name should not appear in store references"
    )
