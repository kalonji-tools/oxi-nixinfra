# Test files

The `File` module inspects files, directories, and symlinks on the target
host. All checks use GNU coreutils commands.

## Check file existence and type

```python
def test_config_exists(host: Fixture[Host]):
    f = host.file("/etc/nixos/configuration.nix")
    assert f.exists()
    assert f.is_file()
```

```python
def test_directory(host: Fixture[Host]):
    assert host.file("/etc/nixos").is_directory()
```

```python
def test_symlink(host: Fixture[Host]):
    f = host.file("/run/current-system")
    assert f.is_symlink()
    assert "/nix/store/" in f.linked_to()
```

## Check permissions and ownership

```python
def test_shadow_permissions(host: Fixture[Host]):
    f = host.file("/etc/shadow")
    assert f.user() == "root"
    assert f.group() == "shadow"
    assert f.mode() == 640
```

## Check file content

```python
def test_hosts_file(host: Fixture[Host]):
    f = host.file("/etc/hosts")
    assert f.contains("localhost")
    content = f.content()
    assert "127.0.0.1" in content
```

## Check file integrity

```python
def test_checksum(host: Fixture[Host]):
    sha = host.file("/etc/os-release").sha256sum()
    assert len(sha) == 64  # valid SHA-256 hex string
```

## List directory contents

```python
def test_nix_profiles(host: Fixture[Host]):
    entries = host.file("/nix/var/nix/profiles").listdir()
    assert "system" in entries
```

## Available methods

| Method | Returns | Description |
|--------|---------|-------------|
| `exists()` | `bool` | `test -e` |
| `is_file()` | `bool` | `test -f` |
| `is_directory()` | `bool` | `test -d` |
| `is_symlink()` | `bool` | `test -L` |
| `is_executable()` | `bool` | `test -x` |
| `is_pipe()` | `bool` | `test -p` |
| `is_socket()` | `bool` | `test -S` |
| `linked_to()` | `str` | `realpath` |
| `user()` | `str` | `stat -Lc %U` |
| `uid()` | `int` | `stat -Lc %u` |
| `group()` | `str` | `stat -Lc %G` |
| `gid()` | `int` | `stat -Lc %g` |
| `mode()` | `int` | `stat -Lc %a` (octal parsed) |
| `size()` | `int` | `stat -Lc %s` |
| `content()` | `str` | `cat` |
| `contains(pattern)` | `bool` | `grep -qs` |
| `md5sum()` | `str` | `md5sum` |
| `sha256sum()` | `str` | `sha256sum` |
| `listdir()` | `list[str]` | `ls -1 -q` |
