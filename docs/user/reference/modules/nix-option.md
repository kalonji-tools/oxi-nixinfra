# NixOption

NixOS option inspector. Constructed via `host.nix_option(path)`.
Queries the running system's evaluated configuration.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `value()` | `object` | The option's evaluated value |
| `exists()` | `bool` | Whether the option path is valid |

## Async

```python
async def test_option(host: Fixture[Host]):
    opt = await host.a.nix_option("services.openssh.enable")
    assert await opt.value() is True
```
