# Host

The `Host` class is the main entry point for interacting with a NixOS system.

## Import

```python
from oxi_nixinfra import Host
from oxitest import Fixture

def test_example(host: Fixture[Host]):
    ...
```

## Sync methods

| Method | Returns | Description |
|--------|---------|-------------|
| `run(*args)` | `CommandResult` | Execute a command |
| `service(name)` | `Service` | Systemd service inspector |
| `file(path)` | `File` | File inspector |
| `user(name)` | `User` | User inspector |
| `nix_package(name)` | `NixPackage` | Nix package inspector |
| `nix_option(path)` | `NixOption` | NixOS option inspector |
| `system_info()` | `SystemInfo` | OS information |

## Async namespace

`host.a` returns an `AsyncHost` proxy. Same methods, same names — all
return coroutines.

```python
async def test_example(host: Fixture[Host]):
    result = await host.a.run("echo", "hello")
    svc = await host.a.service("nix-daemon")
    assert await svc.is_running()
```

## Fixture lifecycle

- The `Host` instance is session-scoped — the same instance is shared
  across all tests.
- Connection is established lazily on the first command, not at
  construction time.
- The SSH backend uses ControlMaster multiplexing for connection reuse.
