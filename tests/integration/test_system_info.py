"""Integration tests for SystemInfo module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


@oxitest.mark.nixos
def test_nixos_version(host: Fixture[Host]):
    version = host.system_info().nixos_version()
    assert isinstance(version, str), (
        "nixos_version() should return str from nixos-version command"
    )
    assert len(version) > 0, (
        "nixos-version command returned empty output — is nixos-version on PATH?"
    )


@oxitest.mark.nixos
def test_system_profile(host: Fixture[Host]):
    profile = host.system_info().system_profile()
    assert profile.startswith("/nix/store/"), (
        "system profile should be a /nix/store/ path"
        " — check readlink /run/current-system parsing"
    )


@oxitest.mark.nixos
def test_generation_count(host: Fixture[Host]):
    count = host.system_info().generation_count()
    assert isinstance(count, int), (
        "generation_count() should parse nix-env --list-generations as int"
    )
    assert count >= 1, (
        "a running NixOS system always has at least one generation"
        " — check nix-env --list-generations output parsing"
    )


@oxitest.mark.nixos
def test_kernel_version(host: Fixture[Host]):
    version = host.system_info().kernel_version()
    assert isinstance(version, str), (
        "kernel_version() should return str from 'uname -r'"
    )
    assert "." in version, (
        "kernel version should be semver-like (e.g. 6.1.0)"
        " — 'uname -r' output may not be parsed correctly"
    )


@oxitest.mark.nixos
def test_arch(host: Fixture[Host]):
    arch = host.system_info().arch()
    assert isinstance(arch, str), "arch() should return str from 'uname -m'"
    assert len(arch) > 0, (
        "'uname -m' returned empty output — check Backend.execute() stdout capture"
    )


@oxitest.mark.nixos
def test_label(host: Fixture[Host]):
    label = host.system_info().label()
    assert isinstance(label, str), (
        "label() should return str from /etc/os-release parsing"
    )
    assert "NixOS" in label, (
        "label() should contain 'NixOS' on a NixOS system"
        " — check /etc/os-release PRETTY_NAME parsing"
    )


@oxitest.mark.nixos
def test_specialisations(host: Fixture[Host]):
    specs = host.system_info().specialisations()
    assert isinstance(specs, list), (
        "specialisations() should return list from"
        " /run/current-system/specialisation/ listing"
    )
