# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

oxi-nixinfra is a NixOS infrastructure testing library, built as an oxitest plugin with a Rust (PyO3) core. It executes commands on local or remote NixOS hosts and inspects system state through typed modules.

NixOS-specific by design — no multi-distro support.

## Commands

```bash
# Enter development shell (provides cargo, python, maturin, just)
devenv shell

# Build the Rust extension (required before running Python tests)
just build

# Run Rust unit tests
just test-rust

# Run Python integration tests (requires just build first)
just test

# Run all static checks (cargo fmt, clippy)
just check

# Full pre-push gate (clean + check + test-rust + build + test)
just preflight

# Format code
just fmt

# Show all available recipes
just
```

## Architecture

### Two-layer design

**Rust layer** (`src/`): PyO3 `cdylib` crate compiled via maturin.
- `command.rs` — `RawOutput`, `CommandResult` struct
- `backend/` — `Backend` trait, `LocalBackend` (tokio::process), `SshBackend` (openssh)
- `host.rs` — `Host`, `AsyncHost`, `HostInner`, connection string parsing, `OnceLock` cache
- `helpers.rs` — `wrap_sync`, `backend_err_to_py`, `extract_args`
- `modules/` — Service, File, User, NixPackage, NixOption, SystemInfo

**Python layer** (`python/oxi_nixinfra/`): Thin shim for oxitest plugin integration.
- `__init__.py` — re-exports all types from the Rust extension
- `_plugin.py` — `oxitest_plugin()` entry point, `HostProvider` (FixtureProvider), `NixosWrapper` (ExecutionWrapper)

### Module pattern

Every module follows a three-layer pattern:
1. **Async core functions** — `pub async fn foo_impl(inner: &HostInner, ...) -> Result<T, BackendError>`
2. **Sync `#[pyclass]`** — calls `wrap_sync(&self.inner, foo_impl(...))`
3. **Async `#[pyclass]`** — clones `Arc<HostInner>`, calls `future_into_py`

### Sync + async API

- `host.service("nix-daemon").is_running()` — sync
- `host.a.service("nix-daemon").is_running()` — async (via `host.a` namespace)

### Design principle

**Shell commands produce raw output. Rust parses it.** No pipes, no `sh -c`, no `grep`. Every `execute()` call is structured args.

## Configuration

oxitest plugin settings in `pyproject.toml`:
```toml
[tool.oxitest]
plugins = ["oxi_nixinfra._plugin"]

[tool.oxitest.plugin_settings.oxi_nixinfra._plugin]
host = "ssh://dell-xps-9640"
```

Config resolution: `OXITEST_HOST` env var > pyproject.toml > `"local://"` default.

## Testing

- **Rust unit tests** (`just test-rust`): Mock `Backend` trait with `MockBackend`, test command output parsing.
- **Python integration tests** (`just test`): Run real commands on local machine. Tests marked `@oxitest.mark.nixos` auto-skip on non-NixOS.
- **CI**: Runs Rust checks + portable integration tests on Ubuntu. NixOS-specific tests skip automatically.

## graphify

This project has a knowledge graph at graphify-out/. Use `graphify query`, `graphify path`, `graphify explain` when graphify-out/graph.json exists.
