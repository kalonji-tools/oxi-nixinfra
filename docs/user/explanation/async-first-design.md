# Async-first design

oxi-nixinfra implements every operation as an `async fn` in Rust. The sync
API wraps the async core — not the other way around.

## How it works

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

Each module has:

1. **Core async functions** — the real implementation, shared between both
   APIs.
2. **Sync wrapper** — a `#[pyclass]` that calls `runtime.block_on(core_fn())`.
3. **Async wrapper** — a `#[pyclass]` that calls
   `pyo3_async_runtimes::tokio::future_into_py()`.

## The `host.a` namespace

The sync `Host` and async `AsyncHost` share the same underlying
`Arc<HostInner>`. The `.a` property returns an `AsyncHost` that provides
async versions of every method:

```python
# Sync
svc = host.service("nix-daemon")
assert svc.is_running()

# Async — same method names
svc = await host.a.service("nix-daemon")
assert await svc.is_running()
```

## Why async-first?

- The SSH backend (`openssh` crate) is inherently async.
- `tokio::process::Command` for local execution is also async.
- Writing the core logic once in async and wrapping it for sync avoids
  code duplication.
- The sync API has no performance penalty — `block_on()` adds negligible
  overhead for command execution workloads.
