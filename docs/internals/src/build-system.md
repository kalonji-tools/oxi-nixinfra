# Build System

## Toolchain

oxi-nixinfra uses **maturin** to compile the Rust crate into a Python
extension module via PyO3.

```
pyproject.toml  →  maturin  →  .so / .pyd  →  Python wheel
Cargo.toml     ↗
```

Key config in `pyproject.toml`:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "python"
module-name = "oxi_nixinfra._oxi_nixinfra"
```

## devenv

The development shell (`devenv.nix`) provides all tools: Rust toolchain,
Python 3.12, maturin, uv, just, ruff, codespell, mdbook.

```bash
devenv shell
```

### Nix shell workarounds

The `justfile` includes workarounds for Nix-injected Python paths that
corrupt maturin's ABI tag detection:

```just
fix_env := if env("IN_NIX_SHELL", "") != "" {
    "unset _PYTHON_SYSCONFIGDATA_NAME PYTHONPATH &&"
} else { "" }
```

`VIRTUAL_ENV` must also be set explicitly for maturin to find the venv.

## Dependency groups

| Group | Contents | Purpose |
|-------|----------|---------|
| `build` | maturin | Building the extension |
| `test` | maturin + oxitest | Running tests |
| `code-quality` | ruff, codespell | Static analysis |
| `docs` | mkdocs-material, plugins | Building documentation |

## CI

GitHub Actions (no devenv). Two parallel jobs:

1. **check** — `just check` (ruff, cargo fmt, clippy, codespell)
2. **test** — `just test-rust`, `just build`, `just test`

Uses `dtolnay/rust-toolchain`, `astral-sh/setup-uv`, `Swatinem/rust-cache`.
NixOS-specific integration tests auto-skip on Ubuntu runners via the
`@oxitest.mark.nixos` marker.
