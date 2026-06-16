# Test Nix packages

The `NixPackage` module checks whether packages are installed in the system
profile and retrieves version and store path information.

## Check if a package is installed

```python
def test_git_installed(host: Fixture[Host]):
    assert host.nix_package("git").is_installed()
```

## Get package version

```python
def test_python_version(host: Fixture[Host]):
    pkg = host.nix_package("python3")
    assert pkg.is_installed()
    assert pkg.version().startswith("3.")
```

## Get Nix store path

```python
def test_store_path(host: Fixture[Host]):
    pkg = host.nix_package("openssh")
    path = pkg.store_path()
    assert path.startswith("/nix/store/")
```

## How it works

oxi-nixinfra queries the system profile using:

```
nix-store -q --references /run/current-system/sw | grep <name>
```

The version is parsed from the store path: `/nix/store/<hash>-<name>-<version>`.

!!! note
    This queries the **system profile** only. User profiles and flake-managed
    packages may not be visible through this module.

## Available methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_installed()` | `bool` | Package in system profile |
| `version()` | `str` | Parsed from store path |
| `store_path()` | `str` | Full `/nix/store/...` path |
