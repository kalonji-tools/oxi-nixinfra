"""Integration tests for SystemInfo module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


@oxitest.mark.nixos
def test_nixos_version(host: Fixture[Host]):
    version = host.system_info().nixos_version()
    assert isinstance(version, str), (
        f"nixos_version() returned {type(version).__name__}, expected str"
    )
    assert len(version) > 0, "nixos_version() returned empty string"


@oxitest.mark.nixos
def test_system_profile(host: Fixture[Host]):
    profile = host.system_info().system_profile()
    assert profile.startswith("/nix/store/"), (
        f"system_profile() should start with /nix/store/, got {profile!r}"
    )


@oxitest.mark.nixos
def test_generation_count(host: Fixture[Host]):
    count = host.system_info().generation_count()
    assert isinstance(count, int), (
        f"generation_count() returned {type(count).__name__}, expected int"
    )
    assert count >= 1, f"generation_count() is {count}, expected at least 1"


@oxitest.mark.nixos
def test_kernel_version(host: Fixture[Host]):
    version = host.system_info().kernel_version()
    assert isinstance(version, str), (
        f"kernel_version() returned {type(version).__name__}, expected str"
    )
    assert "." in version, (
        f"kernel_version() is {version!r}, expected semver-like string"
    )


@oxitest.mark.nixos
def test_arch(host: Fixture[Host]):
    arch = host.system_info().arch()
    assert isinstance(arch, str), f"arch() returned {type(arch).__name__}, expected str"
    assert len(arch) > 0, "arch() returned empty string"


@oxitest.mark.nixos
def test_label(host: Fixture[Host]):
    label = host.system_info().label()
    assert isinstance(label, str), (
        f"label() returned {type(label).__name__}, expected str"
    )
    assert "NixOS" in label, f"label() should contain 'NixOS', got {label!r}"


@oxitest.mark.nixos
def test_specialisations(host: Fixture[Host]):
    specs = host.system_info().specialisations()
    assert isinstance(specs, list), (
        f"specialisations() returned {type(specs).__name__}, expected list"
    )
