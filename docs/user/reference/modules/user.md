# User

User account inspector. Constructed via `host.user(name)`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `exists()` | `bool` | `id <name>` |
| `name()` | `str` | Username |
| `uid()` | `int` | `id -u` |
| `gid()` | `int` | `id -g` |
| `group()` | `str` | `id -ng` |
| `groups()` | `list[str]` | `id -nG` |
| `home()` | `str` | `getent passwd` field 5 |
| `shell()` | `str` | `getent passwd` field 6 |

## Async

All methods are available as coroutines via `host.a.user(name)`:

```python
async def test_user(host: Fixture[Host]):
    u = await host.a.user("root")
    assert await u.exists()
```
