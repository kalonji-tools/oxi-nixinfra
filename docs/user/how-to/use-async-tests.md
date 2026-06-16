# Use async tests

oxi-nixinfra supports async tests through the `host.a` namespace. Every
sync method has an async counterpart with the same name.

## Basic async test

```python
async def test_nix_daemon(host: Fixture[Host]):
    svc = await host.a.service("nix-daemon")
    assert await svc.is_running()
```

## Run a command asynchronously

```python
async def test_echo(host: Fixture[Host]):
    result = await host.a.run("echo", "hello")
    assert result.succeeded()
    assert result.stdout.strip() == "hello"
```

## Multiple async checks

```python
async def test_system(host: Fixture[Host]):
    info = await host.a.system_info()
    assert info.distribution() == "nixos"

    f = await host.a.file("/etc/os-release")
    assert await f.exists()
```

## When to use async

Use async tests when:

- Testing over SSH where network latency matters
- Running multiple independent checks that could benefit from concurrency
- Your test suite already uses async fixtures or patterns

For local testing, sync and async performance is equivalent.
