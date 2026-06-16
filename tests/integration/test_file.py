"""Integration tests for File module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_etc_os_release_exists(host: Fixture[Host]):
    assert host.file("/etc/os-release").exists(), (
        "/etc/os-release missing — is this a Linux system?"
    )


def test_etc_os_release_is_file(host: Fixture[Host]):
    f = host.file("/etc/os-release")
    assert f.is_file(), "/etc/os-release exists but is not a regular file"


def test_etc_is_directory(host: Fixture[Host]):
    assert host.file("/etc").is_directory(), "/etc is not a directory"


def test_nonexistent_file(host: Fixture[Host]):
    assert not host.file("/nonexistent-path-12345").exists(), (
        "fabricated path should not exist on any system"
    )


@oxitest.mark.nixos
def test_file_content(host: Fixture[Host]):
    content = host.file("/etc/os-release").content()
    assert "nixos" in content.lower(), (
        f"/etc/os-release does not mention 'nixos': {content[:200]!r}"
    )


def test_file_user_root(host: Fixture[Host]):
    owner = host.file("/etc/os-release").user()
    assert owner == "root", f"/etc/os-release owned by {owner!r}, expected 'root'"


def test_file_mode(host: Fixture[Host]):
    mode = host.file("/etc/os-release").mode()
    assert isinstance(mode, int), f"mode() returned {type(mode).__name__}, expected int"
    assert mode > 0, f"mode is {oct(mode)}, expected a positive octal value"


@oxitest.mark.nixos
def test_etc_hosts_is_nix_managed(host: Fixture[Host]):
    assert host.file("/etc/hosts").is_nix_managed(), (
        "/etc/hosts should be NixOS-managed on NixOS"
    )


@oxitest.mark.nixos
def test_etc_hosts_store_path(host: Fixture[Host]):
    path = host.file("/etc/hosts").store_path()
    assert path is not None, "/etc/hosts store_path should not be None"
    assert path.startswith("/nix/store/"), (
        f"/etc/hosts store_path should start with /nix/store/, got {path!r}"
    )


def test_nonexistent_file_not_nix_managed(host: Fixture[Host]):
    assert not host.file("/nonexistent-path-12345").is_nix_managed(), (
        "nonexistent file should not be NixOS-managed"
    )
