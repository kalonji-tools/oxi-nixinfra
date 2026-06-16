"""Integration tests for Host.run()."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_run_echo(host: Fixture[Host]):
    result = host.run("echo", "hello")
    assert result.succeeded(), (
        f"'echo hello' exited with rc={result.rc}, stderr={result.stderr!r}"
    )
    assert result.stdout.strip() == "hello", (
        f"stdout was {result.stdout!r}, expected 'hello'"
    )
    assert result.stderr == "", f"unexpected stderr: {result.stderr!r}"


def test_run_false(host: Fixture[Host]):
    result = host.run("false")
    assert result.failed(), f"'false' unexpectedly succeeded with rc={result.rc}"
    assert result.rc != 0, "'false' returned rc=0, expected non-zero"


def test_run_command_recorded(host: Fixture[Host]):
    result = host.run("echo", "a", "b")
    assert result.command == "echo a b", (
        f"recorded command was {result.command!r}, expected 'echo a b'"
    )
