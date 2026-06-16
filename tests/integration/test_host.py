"""Integration tests for Host.run()."""
from oxi_nixinfra import Host
from oxitest import Fixture

def test_run_echo(host: Fixture[Host]):
    result = host.run("echo", "hello")
    assert result.succeeded()
    assert result.stdout.strip() == "hello"
    assert result.stderr == ""

def test_run_false(host: Fixture[Host]):
    result = host.run("false")
    assert result.failed()
    assert result.rc != 0

def test_run_command_recorded(host: Fixture[Host]):
    result = host.run("echo", "a", "b")
    assert result.command == "echo a b"
