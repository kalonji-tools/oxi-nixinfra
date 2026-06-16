# Service

Systemd service inspector. Constructed via `host.service(name)`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_running()` | `bool` | `systemctl is-active` |
| `is_enabled()` | `bool` | `systemctl is-enabled` |
| `exists()` | `bool` | `systemctl list-unit-files` |
| `is_masked()` | `bool` | Masked unit check |
| `is_valid()` | `bool` | `systemd-analyze verify` |
| `properties()` | `dict` | `systemctl show` key-value pairs |

## Async

All methods are available as coroutines via `host.a.service(name)`:

```python
async def test_service(host: Fixture[Host]):
    svc = await host.a.service("nix-daemon")
    assert await svc.is_running()
```
