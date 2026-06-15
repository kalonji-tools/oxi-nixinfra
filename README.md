# oxi-nixinfra — Technical Specification

> NixOS infrastructure testing library. oxitest plugin. Rust core.

## 1. Overview

oxi-nixinfra is a standalone Python package for testing NixOS infrastructure.
It executes commands on local or remote NixOS hosts, inspects system state
through typed modules, and reports results through oxitest's plugin system.

The library is NixOS-specific by design. It does not support other Linux
distributions, BSDs, macOS, or Windows. This constraint eliminates platform
polymorphism and keeps every module focused on a single, well-understood
implementation.

### 1.1 Goals

- Test NixOS system state: services, files, users, packages, options.
- Execute commands locally or over SSH with zero-thought argument safety.
- First-class sync and async APIs with identical method names.
- Rust core for learning, performance, and type safety; thin Python shim
  for oxitest integration.
- Clean, explicit API — methods not properties, structured args not shell
  strings.

### 1.2 Non-Goals

- Multi-distro or cross-platform support.
- Config management tool integration (Ansible, Salt, Puppet).
- Container backends (Docker, Podman, Kubernetes).
- Nix expression evaluation or static analysis (deferred to v0.2.0).

## 2. Architecture

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
│    LocalBackend          duct                         │
│    SshBackend            openssh + tokio              │
│  modules/               system inspection modules    │
│    Service / AsyncService                             │
│    File / AsyncFile                                   │
│    User / AsyncUser                                   │
│    NixPackage / AsyncNixPackage                       │
│    NixOption / AsyncNixOption                         │
│    SystemInfo / AsyncSystemInfo                       │
└─────────────────────────────────────────────────────┘
```

### 2.1 Dependency Stack

| Crate         | Purpose                                   |
|---------------|-------------------------------------------|
| `pyo3`        | Rust ↔ Python bridge                      |
| `openssh`     | SSH command execution, ControlMaster mux  |
| `tokio`       | Async runtime (required by openssh)       |
| `duct`        | Local command execution                   |
| `shlex`       | Shell quoting (SSH remote commands)       |
| `pyo3-async-runtimes` | `future_into_py` for async PyO3   |

### 2.2 Build System

- **maturin** with PyO3 bindings.
- `pyproject.toml` declares Python metadata and maturin build backend.
- `Cargo.toml` declares the `cdylib` crate.
- Single output: a Python wheel containing the compiled extension module
  and the `oxi_nixinfra` Python package.

## 3. Configuration

### 3.1 Plugin Registration

```toml
# pyproject.toml
[tool.oxitest]
plugins = ["oxi_nixinfra"]

[tool.oxitest.plugin_settings.oxi_nixinfra]
host = "ssh://dell-xps-9640"
ssh_config = "~/.ssh/config"
```

### 3.2 Resolution Order

1. `OXITEST_HOST` environment variable (highest priority).
2. `[tool.oxitest.plugin_settings.oxi_nixinfra].host` in `pyproject.toml`.
3. `"local://"` (default).

### 3.3 Connection Strings

```
local://                        # local subprocess
ssh://hostname                  # SSH with defaults
ssh://user@hostname             # SSH with explicit user
ssh://user@hostname:port        # SSH with explicit port
```

## 4. Public API

### 4.1 Host

```python
from oxi_nixinfra import Host
from oxitest import Fixture

def test_something(host: Fixture[Host]):
    result = host.run("echo", "hello")
    assert result.succeeded()
```

#### Sync Methods

| Method                        | Returns          | Description                   |
|-------------------------------|------------------|-------------------------------|
| `run(*args)`                  | `CommandResult`  | Execute command               |
| `service(name)`               | `Service`        | Systemd service inspector     |
| `file(path)`                  | `File`           | File inspector                |
| `user(name=None)`             | `User`           | User inspector                |
| `nix_package(name)`           | `NixPackage`     | Nix package inspector         |
| `nix_option(path)`            | `NixOption`      | NixOS option inspector        |
| `system_info()`               | `SystemInfo`     | OS information                |

#### Async Namespace

`host.a` returns an `AsyncHost` proxy. Same methods, same names, all
return coroutines.

```python
async def test_something(host: Fixture[Host]):
    result = await host.a.run("echo", "hello")
    svc = await host.a.service("nix-daemon")
    assert await svc.is_running()
```

### 4.2 CommandResult

| Member       | Type   | Description                        |
|--------------|--------|------------------------------------|
| `rc`         | `int`  | Exit code                          |
| `stdout`     | `str`  | Standard output (UTF-8 lossy)      |
| `stderr`     | `str`  | Standard error (UTF-8 lossy)       |
| `command`    | `str`  | The command string as executed      |

| Method        | Returns | Description                       |
|---------------|---------|-----------------------------------|
| `succeeded()` | `bool`  | `rc == 0`                         |
| `failed()`    | `bool`  | `rc != 0`                         |

### 4.3 Service Module

Systemd-only. No init system detection.

| Method                  | Returns        | Description                        |
|-------------------------|----------------|------------------------------------|
| `is_running()`          | `bool`         | `systemctl is-active`              |
| `is_enabled()`          | `bool`         | `systemctl is-enabled`             |
| `exists()`              | `bool`         | `systemctl list-unit-files`        |
| `is_masked()`           | `bool`         | Masked unit check                  |
| `is_valid()`            | `bool`         | `systemd-analyze verify`           |
| `properties()`          | `dict`         | `systemctl show` key-value pairs   |

### 4.4 File Module

GNU coreutils-only. No BSD/macOS `stat` variants.

| Method                  | Returns        | Description                        |
|-------------------------|----------------|------------------------------------|
| `exists()`              | `bool`         | `test -e`                          |
| `is_file()`             | `bool`         | `test -f`                          |
| `is_directory()`        | `bool`         | `test -d`                          |
| `is_symlink()`          | `bool`         | `test -L`                          |
| `is_executable()`       | `bool`         | `test -x`                          |
| `is_pipe()`             | `bool`         | `test -p`                          |
| `is_socket()`           | `bool`         | `test -S`                          |
| `linked_to()`           | `str`          | `realpath`                         |
| `user()`                | `str`          | `stat -Lc %U`                     |
| `uid()`                 | `int`          | `stat -Lc %u`                     |
| `group()`               | `str`          | `stat -Lc %G`                     |
| `gid()`                 | `int`          | `stat -Lc %g`                     |
| `mode()`                | `int`          | `stat -Lc %a` (octal parsed)      |
| `size()`                | `int`          | `stat -Lc %s`                     |
| `content()`             | `str`          | `cat`                              |
| `contains(pattern)`     | `bool`         | `grep -qs`                         |
| `md5sum()`              | `str`          | `md5sum`                           |
| `sha256sum()`           | `str`          | `sha256sum`                        |
| `listdir()`             | `list[str]`    | `ls -1 -q`                        |

### 4.5 User Module

Linux-only. No BSD or Windows variants.

| Method                  | Returns           | Description                     |
|-------------------------|-------------------|---------------------------------|
| `exists()`              | `bool`            | `id <name>`                     |
| `name()`                | `str`             | Username                        |
| `uid()`                 | `int`             | `id -u`                         |
| `gid()`                 | `int`             | `id -g`                         |
| `group()`               | `str`             | `id -ng`                        |
| `groups()`              | `list[str]`       | `id -nG`                        |
| `home()`                | `str`             | `getent passwd` field 5         |
| `shell()`               | `str`             | `getent passwd` field 6         |

### 4.6 NixPackage Module

Queries the system profile by default.

| Method                  | Returns        | Description                        |
|-------------------------|----------------|------------------------------------|
| `is_installed()`        | `bool`         | Package in system profile          |
| `version()`             | `str`          | Parsed from store path             |
| `store_path()`          | `str`          | Full `/nix/store/...` path         |

Implementation:
- `nix-store -q --references /run/current-system/sw | grep <name>`
- Version parsed from store path: `/nix/store/<hash>-<name>-<version>`

Note: semantics may evolve. The Nix package model (profiles, closures,
store paths) differs fundamentally from traditional package managers.

### 4.7 NixOption Module

Queries the running system's evaluated configuration.

| Method                  | Returns        | Description                        |
|-------------------------|----------------|------------------------------------|
| `value()`               | `object`       | The option's evaluated value       |
| `exists()`              | `bool`         | Whether the option path is valid   |

Implementation strategy deferred — depends on whether the target system
uses flakes or channels. Candidates: `nixos-option`, `nix eval`,
`nix-instantiate --eval`. Will be resolved during implementation.

### 4.8 SystemInfo Module

Reads `/etc/os-release`. NixOS always has this file.

| Method                  | Returns        | Description                        |
|-------------------------|----------------|------------------------------------|
| `type()`                | `str`          | `"linux"` (always)                 |
| `distribution()`        | `str`          | `"nixos"`                          |
| `release()`             | `str`          | NixOS version (e.g., `"24.05"`)   |
| `codename()`            | `str | None`   | Release codename if present        |
| `arch()`                | `str`          | `uname -m`                         |

## 5. Internal Design

### 5.1 Async-First, Sync Wraps

Every module method is implemented as an `async fn` in Rust. Two `#[pyclass]`
wrappers exist per module:

```
                  ┌──────────────────────────┐
                  │  async fn core logic     │
                  │  (shared, not exported)   │
                  └──────┬──────────┬────────┘
                         │          │
              ┌──────────▼──┐  ┌───▼───────────┐
              │  Service    │  │  AsyncService  │
              │  #[pyclass] │  │  #[pyclass]    │
              │  block_on() │  │  future_into() │
              └─────────────┘  └────────────────┘
```

- `Service` methods call `runtime.block_on(core_fn())`.
- `AsyncService` methods call `pyo3_async_runtimes::tokio::future_into_py()`.
- Logic lives in one place. Wrappers are mechanical.

### 5.2 Host Internals

```rust
struct HostInner {
    backend: Box<dyn Backend>,
    runtime: tokio::runtime::Runtime,
}

#[pyclass]
struct Host {
    inner: Arc<HostInner>,
}

#[pyclass]
struct AsyncHost {
    inner: Arc<HostInner>,
}
```

- `Host` exposes sync module constructors: `service()`, `file()`, etc.
- `AsyncHost` exposes async module constructors with the same names.
- `host.a` (Python `#[getter]`) returns an `AsyncHost` sharing the same
  `Arc<HostInner>`.
- `HostInner` is constructed once via `OnceLock`, cached by connection
  string. `FixtureProvider.create()` always returns the same `Host`.

### 5.3 Backend Trait

```rust
#[async_trait]
trait Backend: Send + Sync {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<RawOutput>;
}
```

- `LocalBackend`: wraps `duct::cmd(program, args).stdout_capture().stderr_capture().unchecked().run()` inside `tokio::task::spawn_blocking`.
- `SshBackend`: wraps `openssh::Session::command(program).args(args).output()`.
- Both return `RawOutput { rc: i32, stdout: Vec<u8>, stderr: Vec<u8> }`.
- `CommandResult` is constructed from `RawOutput` with `String::from_utf8_lossy`.

### 5.4 Backend Selection

Parsed from the connection string at `Host` construction:

| Scheme     | Backend        |
|------------|----------------|
| `local://` | `LocalBackend` |
| `ssh://`   | `SshBackend`   |
| (none)     | `LocalBackend` |

### 5.5 Error Handling

| Failure                        | Behavior                              |
|--------------------------------|---------------------------------------|
| Command returns non-zero       | Normal `CommandResult`, not exception |
| SSH connection refused/timeout | `ConnectionError` raised              |
| SSH session drops mid-run      | `ConnectionError` raised              |
| Connection validation          | Lazy — on first command, not on construction |

### 5.6 No Macro Yet

Sync/async wrapper pairs are written manually for v0.1.0. By module 3-4,
if the pattern is stable, a macro may be extracted. Premature abstraction
is avoided.

## 6. oxitest Integration

### 6.1 Plugin Entry Point

```python
# python/oxi_nixinfra/_plugin.py
from oxitest.plugin import Plugin, FixtureProvider
from oxi_nixinfra import Host
import os


class HostProvider:
    def __init__(self, config):
        self._config = config or {}

    @property
    def name(self) -> str:
        return "oxi-nixinfra:host"

    @property
    def fixture_type(self) -> type:
        return Host

    def create(self, ctx):
        host_str = (
            os.environ.get("OXITEST_HOST")
            or self._config.get("host")
            or "local://"
        )
        ssh_config = self._config.get("ssh_config")
        return Host._from_config(host_str, ssh_config=ssh_config)

    def teardown(self, value):
        pass


def oxitest_plugin(*, config=None) -> Plugin:
    return Plugin(
        fixture_providers=[HostProvider(config)],
    )
```

### 6.2 Fixture Injection

```python
from oxitest import Fixture
from oxi_nixinfra import Host

def test_nix_daemon(host: Fixture[Host]):
    assert host.service("nix-daemon").is_running()
```

- `Fixture[Host]` matches `HostProvider.fixture_type`.
- `create()` returns a cached `Host` via Rust-side `OnceLock`.
- Session-scoped by caching — same `Host` instance for every test.

### 6.3 Async Tests

oxi-nixinfra dogfoods oxitest's `AsyncBackend` protocol. Async tests
use the `host.a` namespace:

```python
from oxitest import Fixture

async def test_nix_daemon(host: Fixture[Host]):
    svc = await host.a.service("nix-daemon")
    assert await svc.is_running()
```

## 7. Project Structure

```
oxi-nixinfra/
├── Cargo.toml
├── pyproject.toml
├── src/
│   ├── lib.rs                    # PyO3 module registration
│   ├── host.rs                   # Host, AsyncHost, HostInner
│   ├── command.rs                # CommandResult, RawOutput
│   ├── backend/
│   │   ├── mod.rs                # Backend trait
│   │   ├── local.rs              # LocalBackend (duct)
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
    ├── unit/                     # Mocked backend tests (Rust)
    └── integration/              # Local backend tests (Python/oxitest)
```

## 8. Testing Strategy

### 8.1 Unit Tests (Rust)

Mock the `Backend` trait to return canned `RawOutput`. Test:
- Connection string parsing.
- Module output parsing (e.g., `systemctl show` → dict).
- `CommandResult` construction and methods.
- Store path version parsing for `NixPackage`.
- `/etc/os-release` parsing for `SystemInfo`.

### 8.2 Integration Tests (Python)

Run against the local NixOS dev machine via `LocalBackend`. Test:
- `host.run()` executes real commands.
- `host.service("nix-daemon").is_running()` returns `True`.
- `host.user("root").exists()` returns `True`.
- `host.file("/etc/os-release").exists()` returns `True`.
- `host.system_info().distribution()` returns `"nixos"`.
- Plugin loads correctly via oxitest.

### 8.3 CI (Deferred)

NixOS VM via `nixos-test` or `microvm.nix` for full SSH backend testing.
Not required for v0.1.0.

## 9. Release Plan

### v0.1.0 — Runtime Testing

Milestone scope:
- `Host` with `local` and `ssh` backends.
- `host.run()` / `CommandResult`.
- Sync + async APIs (`host.a` namespace).
- Modules: `Service`, `File`, `User`, `NixPackage`, `NixOption`, `SystemInfo`.
- oxitest `FixtureProvider` integration.
- Unit + local integration tests.
- PyPI package: `oxi-nixinfra`.

### v0.2.0 — Static Analysis

- `NixConfig` module via `rnix` crate (Nix AST parsing).
- Drift detection: compare `NixOption` (runtime) vs `NixConfig` (declared).

### Future

- Additional modules: `Process`, `Socket`, `MountPoint`, `Sysctl`, `Environment`.
- `NixGeneration` — system generations, rollback targets.
- `NixFlake` — flake metadata inspection.
- NixOS VM CI infrastructure.
- Macro extraction for sync/async boilerplate (if pattern stabilizes).

## 10. Migration from pytest-testinfra

### What Changes

| pytest-testinfra                           | oxi-nixinfra                                |
|--------------------------------------------|---------------------------------------------|
| `import testinfra`                         | `from oxi_nixinfra import Host`             |
| `host = testinfra.get_host("ssh://...")`   | `host: Fixture[Host]` (injected)            |
| `host.run("cmd %s", arg)`                  | `host.run("cmd", arg)`                      |
| `host.service("x").is_running` (property)  | `host.service("x").is_running()` (method)   |
| `host.file("/x").exists` (property)        | `host.file("/x").exists()` (method)         |
| `pytest.ini` / `conftest.py` config        | `pyproject.toml` plugin settings            |
| `--host` CLI flag                          | `OXITEST_HOST` env var or config            |

### What Stays the Same

- Module names: `service`, `file`, `user`.
- Method names: `is_running`, `exists`, `is_enabled`, etc.
- Connection string format: `local://`, `ssh://user@host`.
- Test structure: one assertion per system property.

## 11. Open Questions

1. **NixOption implementation**: Which command (`nixos-option`, `nix eval`,
   `nix-instantiate --eval`) works reliably across flake and channel
   systems? Resolve during implementation.
2. **NixPackage profile parameter**: Exact semantics for querying user
   profiles vs system profile. May need exploration.
3. **openssh async + duct sync mismatch**: `duct` is sync-only.
   `LocalBackend` wraps duct calls in `tokio::task::spawn_blocking`.
   Verify this doesn't cause issues under load.
4. **pyo3-async-runtimes maturity**: Confirm `future_into_py` works
   reliably with oxitest's `AsyncBackend` protocol.
