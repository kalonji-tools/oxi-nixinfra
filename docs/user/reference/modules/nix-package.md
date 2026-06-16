# NixPackage

Nix package inspector. Constructed via `host.nix_package(name)`.
Queries the system profile.

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_installed()` | `bool` | Package in system profile |
| `version()` | `str` | Parsed from store path |
| `store_path()` | `str` | Full `/nix/store/...` path |

## How it works

Queries the system profile using:

```
nix-store -q --references /run/current-system/sw | grep <name>
```

Version is parsed from the store path: `/nix/store/<hash>-<name>-<version>`.

## Async

```python
async def test_package(host: Fixture[Host]):
    pkg = await host.a.nix_package("git")
    assert await pkg.is_installed()
```
