# Introduction

Welcome to the oxi-nixinfra internals book. This guide is for contributors
who want to understand, modify, or extend the codebase.

## Prerequisites

- Rust (stable toolchain)
- Python 3.12+
- [devenv](https://devenv.sh/) (recommended) or manual tool installation
- Familiarity with PyO3 basics

## Quick start

```bash
# Enter development shell (installs all tools)
devenv shell

# Build the extension
just build

# Run Rust unit tests
just test-rust

# Run Python integration tests
just test

# Run all static checks
just check

# Full pre-push gate
just preflight
```

## Project structure

```
oxi-nixinfra/
├── Cargo.toml
├── pyproject.toml
├── src/
│   ├── lib.rs                    # PyO3 module registration
│   ├── host.rs                   # Host, AsyncHost, HostInner
│   ├── command.rs                # CommandResult, RawOutput
│   ├── helpers.rs                # wrap_sync, backend_err_to_py, extract_args
│   ├── backend/
│   │   ├── mod.rs                # Backend trait
│   │   ├── local.rs              # LocalBackend (tokio::process)
│   │   └── ssh.rs                # SshBackend (openssh)
│   └── modules/
│       ├── mod.rs
│       ├── service.rs            # Service + AsyncService
│       ├── file.rs               # File + AsyncFile
│       ├── user.rs               # User + AsyncUser
│       ├── nix_package.rs        # NixPackage + AsyncNixPackage
│       ├── nix_option.rs         # NixOption + AsyncNixOption
│       └── system_info.rs        # SystemInfo + AsyncSystemInfo
├── python/
│   └── oxi_nixinfra/
│       ├── __init__.py           # Re-exports from Rust module
│       └── _plugin.py            # oxitest_plugin() + HostProvider
└── tests/
    └── integration/              # Python integration tests
```
