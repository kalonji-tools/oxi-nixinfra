# File

File, directory, and symlink inspector. Constructed via `host.file(path)`.
Uses GNU coreutils commands.

## Methods

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

## Async

All methods are available as coroutines via `host.a.file(path)`:

```python
async def test_file(host: Fixture[Host]):
    f = await host.a.file("/etc/os-release")
    assert await f.exists()
```
