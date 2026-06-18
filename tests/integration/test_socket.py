"""Integration tests for Socket module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_socket_protocol_tcp(host: Fixture[Host]):
    assert host.socket("tcp://0.0.0.0:22").protocol() == "tcp", (
        "protocol() should parse 'tcp' from the spec URI"
    )


def test_socket_protocol_unix(host: Fixture[Host]):
    assert host.socket("unix:///var/run/test.sock").protocol() == "unix", (
        "protocol() should parse 'unix' from the spec URI"
    )


def test_socket_address_tcp(host: Fixture[Host]):
    assert host.socket("tcp://127.0.0.1:80").address() == "127.0.0.1", (
        "address() should parse the host from tcp spec"
    )


def test_socket_port_tcp(host: Fixture[Host]):
    assert host.socket("tcp://0.0.0.0:443").port() == 443, (
        "port() should parse the port number from tcp spec"
    )


def test_socket_port_unix_is_none(host: Fixture[Host]):
    assert host.socket("unix:///var/run/test.sock").port() is None, (
        "unix sockets have no port — should return None"
    )
