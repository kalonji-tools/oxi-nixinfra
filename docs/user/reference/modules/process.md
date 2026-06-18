# Process

Process table inspector. Constructed via `host.process()`.
Queries the process table via `ps`.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `list()` | `list[dict]` | All processes. Keys: `pid`, `user`, `comm`, `args` |
| `filter(user=None, comm=None)` | `list[dict]` | Filter by user and/or command name |
| `exists(comm)` | `bool` | Whether any process with this name is running |
| `pids(comm)` | `list[int]` | PIDs of processes matching command name |
| `count(comm)` | `int` | Count of matching processes |

## Async

```python
async def test_process(host: Fixture[Host]):
    proc = await host.a.process()
    assert await proc.exists("systemd")
```
