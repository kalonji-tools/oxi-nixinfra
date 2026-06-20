"""Integration tests for the async API (host.a namespace)."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


async def test_async_run_echo(host: Fixture[Host]):
    result = await host.a.run("echo", "hello")
    assert result.succeeded(), "async run() should execute commands — echo failed"
    assert result.stdout.strip() == "hello", (
        "async run() stdout should match sync run() — got different output"
    )


async def test_async_file_exists(host: Fixture[Host]):
    exists = await host.a.file("/etc/os-release").exists()
    assert exists, "async file().exists() should detect /etc/os-release"


async def test_async_file_content(host: Fixture[Host]):
    content = await host.a.file("/etc/os-release").content()
    assert "ID=" in content, (
        "async file().content() should return file contents matching sync variant"
    )


@oxitest.mark.nixos
async def test_async_service_is_running(host: Fixture[Host]):
    running = await host.a.service("nix-daemon").is_running()
    assert running, "async service().is_running() should detect nix-daemon on NixOS"


async def test_async_user_uid(host: Fixture[Host]):
    uid = await host.a.user("root").uid()
    assert uid == 0, "async user().uid() should return 0 for root"


async def test_async_user_exists(host: Fixture[Host]):
    assert await host.a.user("root").exists(), (
        "async user().exists() should detect root user"
    )
    assert not await host.a.user("nonexistent-user-12345").exists(), (
        "async user().exists() should return False for fabricated user"
    )


@oxitest.mark.nixos
async def test_async_sysctl_value(host: Fixture[Host]):
    hostname = await host.a.sysctl("kernel.hostname").value()
    assert isinstance(hostname, str), "async sysctl().value() should return str"
    assert len(hostname) > 0, "kernel.hostname should not be empty"


async def test_async_environment_exists(host: Fixture[Host]):
    assert await host.a.environment().exists("PATH"), (
        "async environment().exists() should detect PATH"
    )
