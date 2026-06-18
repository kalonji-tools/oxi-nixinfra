# MountPoint

Mount point inspector. Constructed via `host.mountpoint(path)`.
Uses `findmnt` for mount information.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `exists()` | `bool` | Whether the path is a mount point |
| `filesystem()` | `str` | Filesystem type (e.g., `ext4`, `btrfs`) |
| `device()` | `str` | Source device |
| `options()` | `list[str]` | Mount options |

## Async

```python
async def test_mount(host: Fixture[Host]):
    mnt = await host.a.mountpoint("/")
    assert await mnt.exists()
    assert await mnt.filesystem() in ("ext4", "btrfs", "tmpfs")
```
