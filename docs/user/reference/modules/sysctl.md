# Sysctl

Kernel parameter inspector. Constructed via `host.sysctl(key)`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `value()` | `str` | `sysctl -n <key>`, trimmed |
| `exists()` | `bool` | Whether the key exists |

## Async

```python
async def test_sysctl(host: Fixture[Host]):
    s = await host.a.sysctl("net.ipv4.ip_forward")
    assert await s.value() in ("0", "1")
```
