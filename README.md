# oxi-nixinfra

[![CI](https://github.com/kalonji-tools/oxi-nixinfra/actions/workflows/ci.yml/badge.svg)](https://github.com/kalonji-tools/oxi-nixinfra/actions/workflows/ci.yml)

> NixOS infrastructure testing library — [oxitest](https://github.com/kalonji-tools/oxitest) plugin, Rust core.

## Quick example

```python
from oxitest import Fixture
from oxi_nixinfra import Host

def test_nix_daemon(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.is_running()
    assert svc.is_enabled()

def test_os_release(host: Fixture[Host]):
    assert host.file("/etc/os-release").exists()
    assert host.system_info().distribution() == "nixos"
```

## Install

```bash
pip install oxi-nixinfra
```

## Configure

```toml
# pyproject.toml
[tool.oxitest]
plugins = ["oxi_nixinfra"]

[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://your-nixos-host"  # or "local://" (default)
```

## Run

```bash
oxitest tests/
```

## Modules

| Module | Description |
|--------|-------------|
| `host.service(name)` | Systemd service inspection |
| `host.file(path)` | File, directory, symlink inspection |
| `host.user(name)` | User account inspection |
| `host.nix_package(name)` | Nix store package queries |
| `host.system_info()` | OS release information |
| `host.environment(name)` | Environment variable inspection |
| `host.mountpoint(path)` | Mount point inspection |
| `host.process()` | Running process inspection |
| `host.socket(spec)` | Network socket inspection |
| `host.sysctl(key)` | Kernel parameter inspection |
| `host.run(*args)` | Raw command execution |

All modules have async counterparts via `host.a`.

## Documentation

- **[User docs](https://kalonji-tools.github.io/oxi-nixinfra/)** — tutorials, how-tos, reference
- **[Internals](https://kalonji-tools.github.io/oxi-nixinfra/internals/)** — contributor guide

## License

[MIT](LICENSE)
