# SystemInfo

OS information inspector. Constructed via `host.system_info()`.
Reads `/etc/os-release`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `type()` | `str` | `"linux"` (always) |
| `distribution()` | `str` | `"nixos"` |
| `release()` | `str` | NixOS version (e.g., `"24.05"`) |
| `codename()` | `str \| None` | Release codename if present |
| `arch()` | `str` | `uname -m` |

## Async

```python
async def test_system(host: Fixture[Host]):
    info = await host.a.system_info()
    assert info.distribution() == "nixos"
```
