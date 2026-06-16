# Getting started

This tutorial walks you through writing your first NixOS infrastructure test
with oxi-nixinfra.

## Prerequisites

- A NixOS machine (local or remote via SSH)
- Python 3.12+
- [oxitest](https://github.com/kalonji-tools/oxitest) installed

## Install oxi-nixinfra

```bash
pip install oxi-nixinfra
```

## Configure your project

Create or update `pyproject.toml` to register the plugin:

```toml
[tool.oxitest]
plugins = ["oxi_nixinfra"]
```

By default, oxi-nixinfra tests against the local machine. To test a remote
host via SSH, add:

```toml
[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://your-nixos-host"
```

## Write your first test

Create a file `tests/test_infra.py`:

```python
from oxitest import Fixture
from oxi_nixinfra import Host


def test_nix_daemon_is_running(host: Fixture[Host]):
    """The nix-daemon service should be active."""
    svc = host.service("nix-daemon")
    assert svc.is_running()


def test_os_release_exists(host: Fixture[Host]):
    """/etc/os-release should exist on every NixOS system."""
    assert host.file("/etc/os-release").exists()


def test_system_is_nixos(host: Fixture[Host]):
    """Verify we're running on NixOS."""
    info = host.system_info()
    assert info.distribution() == "nixos"
```

## Run the tests

```bash
oxitest tests/test_infra.py
```

You should see output like:

```
tests/test_infra.py::test_nix_daemon_is_running  PASSED
tests/test_infra.py::test_os_release_exists       PASSED
tests/test_infra.py::test_system_is_nixos          PASSED

3 passed in 0.42s
```

## What happened

1. oxitest loaded the `oxi_nixinfra` plugin from your `pyproject.toml`.
2. The plugin created a `Host` fixture connected to `local://` (your machine).
3. Each test received the `Host` via `Fixture[Host]` type annotation.
4. The `Host` executed real commands on your NixOS system and returned structured results.

## Next steps

- [Configure a remote SSH connection](../how-to/configure-connection.md)
- [Test systemd services](../how-to/test-services.md)
- [Test files and permissions](../how-to/test-files.md)
- [Use async tests](../how-to/use-async-tests.md)
