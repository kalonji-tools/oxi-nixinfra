# CommandResult

Returned by `host.run()`. Represents the result of executing a command.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `rc` | `int` | Exit code |
| `stdout` | `str` | Standard output (UTF-8 lossy) |
| `stderr` | `str` | Standard error (UTF-8 lossy) |
| `command` | `str` | The command string as executed |

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `succeeded()` | `bool` | `rc == 0` |
| `failed()` | `bool` | `rc != 0` |

## Example

```python
def test_echo(host: Fixture[Host]):
    result = host.run("echo", "hello", "world")
    assert result.succeeded()
    assert result.stdout.strip() == "hello world"
    assert result.rc == 0
```

## Error handling

Non-zero exit codes do **not** raise exceptions. Check `succeeded()` or
`rc` explicitly:

```python
def test_missing_command(host: Fixture[Host]):
    result = host.run("nonexistent-command")
    assert result.failed()
    assert result.rc == 127
```

SSH connection failures (refused, timeout, dropped) raise `ConnectionError`.
