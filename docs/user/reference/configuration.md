# Configuration

## Plugin registration

Register oxi-nixinfra as an oxitest plugin in `pyproject.toml`:

```toml
[tool.oxitest]
plugins = ["oxi_nixinfra"]
```

## Plugin settings

```toml
[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://hostname"
ssh_config = "~/.ssh/config"
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | `str` | `"local://"` | Connection string |
| `ssh_config` | `str` | `None` | Path to SSH config file |

## Connection strings

| Format | Backend | Description |
|--------|---------|-------------|
| `local://` | Local | Execute commands as subprocesses |
| `ssh://hostname` | SSH | Connect with default user and port |
| `ssh://user@hostname` | SSH | SSH with explicit user |
| `ssh://user@hostname:port` | SSH | SSH with explicit user and port |

## Resolution order

1. `OXITEST_HOST` environment variable (highest priority)
2. `[tool.oxitest.plugin_settings.oxi_nixinfra].host` in `pyproject.toml`
3. `"local://"` (default)

## NixOS marker

The plugin registers a `nixos` marker. Tests decorated with
`@oxitest.mark.nixos` are automatically skipped on non-NixOS systems.

```python
import oxitest

@oxitest.mark.nixos
def test_nix_specific(host: Fixture[Host]):
    ...
```
