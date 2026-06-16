# Configure connection

oxi-nixinfra connects to NixOS hosts via connection strings. This guide
covers the available backends and configuration options.

## Connection string formats

| Format | Backend | Description |
|--------|---------|-------------|
| `local://` | Local | Execute commands as subprocesses (default) |
| `ssh://hostname` | SSH | Connect via SSH with default user and port |
| `ssh://user@hostname` | SSH | SSH with explicit user |
| `ssh://user@hostname:port` | SSH | SSH with explicit user and port |

## Configuration methods

### pyproject.toml (recommended)

```toml
[tool.oxitest]
plugins = ["oxi_nixinfra"]

[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://my-nixos-server"
ssh_config = "~/.ssh/config"
```

### Environment variable

```bash
OXITEST_HOST=ssh://my-nixos-server oxitest tests/
```

### Resolution order

1. `OXITEST_HOST` environment variable (highest priority)
2. `[tool.oxitest.plugin_settings.oxi_nixinfra].host` in `pyproject.toml`
3. `local://` (default)

## SSH configuration

The SSH backend uses your system's `ssh` binary and respects `~/.ssh/config`.
ControlMaster multiplexing is used automatically for connection reuse.

To use a custom SSH config file:

```toml
[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://my-server"
ssh_config = "/path/to/ssh_config"
```

## Testing different hosts

Override the host per test run using the environment variable:

```bash
# Test against staging
OXITEST_HOST=ssh://staging oxitest tests/

# Test against production
OXITEST_HOST=ssh://production oxitest tests/

# Test locally
OXITEST_HOST=local:// oxitest tests/
```
