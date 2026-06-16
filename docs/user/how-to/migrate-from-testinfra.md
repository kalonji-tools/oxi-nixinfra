# Migrate from pytest-testinfra

oxi-nixinfra replaces pytest-testinfra for NixOS infrastructure testing.
This guide covers the key differences and how to migrate your tests.

## What changes

| pytest-testinfra | oxi-nixinfra |
|---|---|
| `import testinfra` | `from oxi_nixinfra import Host` |
| `host = testinfra.get_host("ssh://...")` | `host: Fixture[Host]` (injected) |
| `host.run("cmd %s", arg)` | `host.run("cmd", arg)` |
| `host.service("x").is_running` (property) | `host.service("x").is_running()` (method) |
| `host.file("/x").exists` (property) | `host.file("/x").exists()` (method) |
| `pytest.ini` / `conftest.py` config | `pyproject.toml` plugin settings |
| `--host` CLI flag | `OXITEST_HOST` env var or config |

## What stays the same

- Module names: `service`, `file`, `user`
- Method names: `is_running`, `exists`, `is_enabled`, etc.
- Connection string format: `local://`, `ssh://user@host`
- Test structure: one assertion per system property

## Step-by-step migration

### 1. Replace imports

Before:

```python
import testinfra

def test_service(host):
    assert host.service("sshd").is_running
```

After:

```python
from oxitest import Fixture
from oxi_nixinfra import Host

def test_service(host: Fixture[Host]):
    assert host.service("sshd").is_running()
```

### 2. Add parentheses to property access

testinfra uses properties. oxi-nixinfra uses methods. Add `()` to every
property access:

- `svc.is_running` -> `svc.is_running()`
- `f.exists` -> `f.exists()`
- `f.is_file` -> `f.is_file()`
- `u.uid` -> `u.uid()`

### 3. Fix `host.run()` format strings

testinfra uses `%s` formatting. oxi-nixinfra uses structured arguments:

Before:

```python
result = host.run("grep %s %s", pattern, filename)
```

After:

```python
result = host.run("grep", pattern, filename)
```

### 4. Update configuration

Remove testinfra config from `conftest.py` and `pytest.ini`. Add to
`pyproject.toml`:

```toml
[tool.oxitest]
plugins = ["oxi_nixinfra"]

[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://your-host"
```

### 5. Run with oxitest

Replace `pytest` with `oxitest`:

```bash
oxitest tests/
```
