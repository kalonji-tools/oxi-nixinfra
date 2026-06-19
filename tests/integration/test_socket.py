"""Integration tests for Socket module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_socket_protocol_tcp(host: Fixture[Host]):
    assert host.socket("tcp://0.0.0.0:22").protocol() == "tcp", (
        "protocol() should parse 'tcp' from the spec URI"
    )


@oxitest.mark.nixos
def test_nix_daemon_socket_is_listening(host: Fixture[Host]):
    assert host.socket("unix:///nix/var/nix/daemon-socket/socket").is_listening(), (
        "nix-daemon unix socket should be listening on any NixOS system"
    )


def test_nonexistent_port_not_listening(host: Fixture[Host]):
    assert not host.socket("tcp://127.0.0.1:59999").is_listening(), (
        "fabricated high port should not have any listener"
    )
