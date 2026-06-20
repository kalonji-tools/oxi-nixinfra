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


def test_socket_address_tcp(host: Fixture[Host]):
    addr = host.socket("tcp://0.0.0.0:22").address()
    assert addr == "0.0.0.0", (
        "address() should extract the host part from 'tcp://host:port' spec"
    )


def test_socket_port_tcp(host: Fixture[Host]):
    port = host.socket("tcp://0.0.0.0:22").port()
    assert port == 22, "port() should extract the port number from the spec URI"


@oxitest.mark.nixos
def test_socket_address_unix(host: Fixture[Host]):
    addr = host.socket("unix:///nix/var/nix/daemon-socket/socket").address()
    assert addr == "/nix/var/nix/daemon-socket/socket", (
        "address() should return the full path for unix:// sockets"
    )


@oxitest.mark.nixos
def test_socket_port_unix(host: Fixture[Host]):
    port = host.socket("unix:///nix/var/nix/daemon-socket/socket").port()
    assert port is None, "port() should return None for unix sockets — no port concept"


def test_socket_protocol_udp(host: Fixture[Host]):
    assert host.socket("udp://0.0.0.0:53").protocol() == "udp", (
        "protocol() should parse 'udp' from 'udp://' spec URI"
    )
