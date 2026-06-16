"""Integration tests for User module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_root_exists(host: Fixture[Host]):
    assert host.user("root").exists(), "'id root' failed — root user missing?"


def test_root_uid(host: Fixture[Host]):
    uid = host.user("root").uid()
    assert uid == 0, f"root uid is {uid}, expected 0"


def test_root_home(host: Fixture[Host]):
    home = host.user("root").home()
    assert home == "/root", f"root home is {home!r}, expected '/root'"


def test_nonexistent_user(host: Fixture[Host]):
    assert not host.user("nonexistent-user-12345").exists(), (
        "fabricated user name should not resolve via 'id'"
    )


def test_current_user(host: Fixture[Host]):
    name = host.user().name()
    assert isinstance(name, str), f"name() returned {type(name).__name__}, expected str"
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
