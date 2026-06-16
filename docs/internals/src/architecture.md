# Architecture

## HostInner

All state lives in `HostInner`, shared via `Arc`:

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

`host.a` (Python `#[getter]`) returns an `AsyncHost` sharing the same
`Arc<HostInner>`. The `HostInner` is constructed once via `OnceLock`, cached
by connection string. `FixtureProvider.create()` always returns the same
`Host`.

## Backend trait

```rust
#[async_trait]
trait Backend: Send + Sync {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<RawOutput>;
}
```

Two implementations:

- **`LocalBackend`** — `tokio::process::Command` for local subprocess execution.
- **`SshBackend`** — `openssh::Session::command()` with ControlMaster multiplexing.

Both return `RawOutput { rc: i32, stdout: Vec<u8>, stderr: Vec<u8> }`.
`CommandResult` is constructed from `RawOutput` with `String::from_utf8_lossy`.

## Backend selection

Parsed from the connection string at `Host` construction:

| Scheme | Backend |
|--------|---------|
| `local://` | `LocalBackend` |
| `ssh://` | `SshBackend` |
| (none) | `LocalBackend` |

## Error handling

| Failure | Behavior |
|---------|----------|
| Command returns non-zero | Normal `CommandResult`, not exception |
| SSH connection refused/timeout | `ConnectionError` raised |
| SSH session drops mid-run | `ConnectionError` raised |
| Connection validation | Lazy — on first command, not on construction |

## Module pattern

Every module follows a three-layer pattern. See [Adding a Module](adding-a-module.md)
for details.
