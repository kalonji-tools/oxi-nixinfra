# oxi-nixinfra

**NixOS infrastructure testing library — oxitest plugin, Rust core.**

oxi-nixinfra lets you write infrastructure tests for NixOS systems. It
executes commands on local or remote hosts and inspects system state through
typed modules — services, files, users, packages, and NixOS options.

## Quick example

```python
from oxitest import Fixture
from oxi_nixinfra import Host

def test_nix_daemon(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.is_running()
    assert svc.is_enabled()

def test_openssh(host: Fixture[Host]):
    assert host.file("/etc/ssh/sshd_config").exists()
    assert host.user("root").exists()
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
```

## Run

```bash
oxitest tests/
```

## Learn more

- [Getting started](tutorials/getting-started.md) — your first infrastructure test
- [How-to guides](how-to/configure-connection.md) — task-oriented recipes
- [Reference](reference/configuration.md) — complete API documentation
- [Explanation](explanation/why-nixos-only.md) — design decisions and architecture
