"""Integration tests for User module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_root_exists(host: Fixture[Host]):
    assert host.user("root").exists(), "'id root' failed — root user missing?"


def test_root_uid(host: Fixture[Host]):
    uid = host.user("root").uid()
    assert uid == 0, (
        "root is always uid 0 on Linux — check 'id -u' output parsing in user_impl"
    )


def test_root_home(host: Fixture[Host]):
    home = host.user("root").home()
    assert home == "/root", (
        "root home is conventionally /root — check getent passwd field extraction"
    )


def test_nonexistent_user(host: Fixture[Host]):
    assert not host.user("nonexistent-user-12345").exists(), (
        "fabricated user name should not resolve via 'id'"
    )


def test_current_user(host: Fixture[Host]):
    name = host.user().name()
    assert isinstance(name, str), "name() should return str from 'id -nu'"
    assert len(name) > 0, "'id -nu' returned an empty username"


@oxitest.mark.nixos
def test_root_is_declared(host: Fixture[Host]):
    assert host.user("root").is_declared(), (
        "root should be declared in NixOS users-groups.json"
    )


def test_nonexistent_user_not_declared(host: Fixture[Host]):
    assert not host.user("nonexistent-user-12345").is_declared(), (
        "fabricated user should not be in NixOS users-groups.json"
    )


def test_root_gid(host: Fixture[Host]):
    gid = host.user("root").gid()
    assert gid == 0, (
        "root is always gid 0 on Linux — check getent passwd field 3 extraction"
    )


def test_root_group(host: Fixture[Host]):
    group = host.user("root").group()
    assert group == "root", (
        "root's primary group is 'root' — check 'id -gn' or getent group parsing"
    )


def test_root_groups(host: Fixture[Host]):
    groups = host.user("root").groups()
    assert isinstance(groups, list), "groups() should return a list of group names"
    assert len(groups) >= 1, "root should belong to at least one group"
    assert "root" in groups, "root should be a member of the 'root' group"


def test_root_shell(host: Fixture[Host]):
    shell = host.user("root").shell()
    assert shell.startswith("/"), (
        "shell() should return an absolute path — check getent passwd field 6"
    )
    assert "sh" in shell.split("/")[-1], (
        "root's shell should contain 'sh'"
        " (e.g., /bin/bash, /bin/sh, /run/current-system/sw/bin/bash)"
    )
