"""Integration tests for Host.run()."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_run_echo(host: Fixture[Host]):
    result = host.run("echo", "hello")
    assert result.succeeded(), (
        "Host.run() cannot execute basic commands — check LocalBackend or tokio runtime"
    )
    assert result.stdout.strip() == "hello", (
        "stdout mangled during RawOutput → CommandResult conversion"
    )
    assert result.stderr == "", (
        "echo produced stderr — shell environment may be injecting warnings"
    )


def test_run_false(host: Fixture[Host]):
    result = host.run("false")
    assert result.failed(), (
        "failed() should be True when process exits non-zero"
        " — check CommandResult.failed() logic"
    )
    assert result.rc != 0, (
        "false exited 0 — process exit code not propagated through Backend.execute()"
    )


def test_run_command_recorded(host: Fixture[Host]):
    result = host.run("echo", "a", "b")
    assert result.command == "echo a b", (
        "CommandResult.command not joining args correctly"
        " — check RawOutput → CommandResult conversion"
    )
