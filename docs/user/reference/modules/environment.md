# Environment

Environment variable inspector. Constructed via `host.environment()`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `get(name)` | `str \| None` | Value of the variable, or `None` if unset |
| `exists(name)` | `bool` | Whether the variable is set |

## Async

```python
async def test_env(host: Fixture[Host]):
    env = await host.a.environment()
    assert await env.exists("PATH")
```
