# Architecture

oxi-nixinfra is a two-layer system: a Rust core compiled as a Python
extension via PyO3, and a thin Python shim for oxitest plugin integration.

## Layer diagram

```
┌─────────────────────────────────────────────────────┐
│ Python                                              │
│  oxi_nixinfra/                                      │
│    __init__.py          re-exports from Rust         │
│    _plugin.py           oxitest_plugin() entry point │
├─────────────────────────────────────────────────────┤
│ Rust (PyO3 / maturin)                               │
│  Host / AsyncHost       sync & async entry points    │
│  CommandResult          rc, stdout, stderr, command   │
│  backend/               Backend trait + impls         │
│    LocalBackend          tokio::process               │
│    SshBackend            openssh + tokio              │
│  modules/               system inspection modules    │
│    Service, File, User, NixPackage,                  │
│    SystemInfo                                        │
└─────────────────────────────────────────────────────┘
```

## Why Rust?

- **Type safety** — module output parsing is done in Rust with strong
  types, catching errors at compile time.
- **Performance** — command execution and output parsing have minimal
  overhead.
- **Learning** — oxi-nixinfra is part of the oxitest ecosystem, which
  uses Rust as its core language.

## Design principle

**Shell commands produce raw output. Rust parses it.**

Every command executed on the target host uses structured arguments — no
pipes, no `sh -c`, no shell interpolation. The raw bytes come back to Rust,
which parses them into typed values.

## Dependency stack

| Crate | Purpose |
|-------|---------|
| `pyo3` | Rust-Python bridge |
| `openssh` | SSH command execution, ControlMaster mux |
| `tokio` | Async runtime |
| `shlex` | Shell quoting for SSH remote commands |
| `pyo3-async-runtimes` | `future_into_py` for async PyO3 |

## Build system

oxi-nixinfra is compiled with **maturin** using PyO3 bindings. The output
is a Python wheel containing the compiled extension module and the
`oxi_nixinfra` Python package.
