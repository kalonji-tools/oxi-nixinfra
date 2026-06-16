# Test users

The `User` module inspects Linux user accounts on the target host.

## Check if a user exists

```python
def test_root_exists(host: Fixture[Host]):
    assert host.user("root").exists()
```

## Inspect user properties

```python
def test_nixbld_user(host: Fixture[Host]):
    u = host.user("nixbld1")
    assert u.exists()
    assert u.uid() > 0
    assert "nixbld" in u.groups()
```

## Check home directory and shell

```python
def test_root_home(host: Fixture[Host]):
    u = host.user("root")
    assert u.home() == "/root"
    assert u.shell() in ("/bin/bash", "/run/current-system/sw/bin/bash")
```

## Available methods

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
