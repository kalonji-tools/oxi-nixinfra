# Socket

Socket listener inspector. Constructed via `host.socket(spec)`.
Spec format: `tcp://address:port`, `udp://address:port`, or `unix:///path`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_listening()` | `bool` | Whether the socket is in LISTEN state (`ss`) |
| `protocol()` | `str` | `tcp`, `udp`, or `unix` |
| `address()` | `str` | Address or path |
| `port()` | `int \| None` | Port number, `None` for Unix sockets |

## Async

```python
async def test_socket(host: Fixture[Host]):
    sock = await host.a.socket("tcp://0.0.0.0:22")
    assert await sock.is_listening()
```
