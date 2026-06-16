# Adding a Module

This guide walks through adding a new inspection module to oxi-nixinfra.
We'll use a hypothetical `Process` module as an example.

## The three-layer pattern

Every module has three layers:

1. **Core async functions** — `pub async fn` that take `&HostInner` and
   return `Result<T, BackendError>`.
2. **Sync `#[pyclass]`** — wraps core functions with `wrap_sync()`.
3. **Async `#[pyclass]`** — wraps core functions with `future_into_py()`.

## Step 1: Create the module file

Create `src/modules/process.rs`:

```rust
use crate::backend::BackendError;
use crate::helpers::{backend_err_to_py, extract_args, wrap_sync};
use crate::host::HostInner;
use pyo3::prelude::*;
use std::sync::Arc;

// ── Core async functions ─────────────────────────────────────

pub async fn exists_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let output = inner.backend.execute("pgrep", &["-x", name]).await?;
    Ok(output.rc == 0)
}

pub async fn pid_impl(inner: &HostInner, name: &str) -> Result<i32, BackendError> {
    let output = inner.backend.execute("pgrep", &["-x", "-o", name]).await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .parse()
        .map_err(|_| BackendError::ParseError(format!("invalid pid: {stdout}")))
}

// ── Sync #[pyclass] ─────────────────────────────────────────

#[pyclass]
pub struct Process {
    inner: Arc<HostInner>,
    name: String,
}

#[pymethods]
impl Process {
    pub fn exists(&self) -> PyResult<bool> {
        wrap_sync(&self.inner, exists_impl(&self.inner, &self.name))
            .map_err(backend_err_to_py)
    }

    pub fn pid(&self) -> PyResult<i32> {
        wrap_sync(&self.inner, pid_impl(&self.inner, &self.name))
            .map_err(backend_err_to_py)
    }
}

// ── Async #[pyclass] ────────────────────────────────────────

#[pyclass]
pub struct AsyncProcess {
    inner: Arc<HostInner>,
    name: String,
}

#[pymethods]
impl AsyncProcess {
    pub fn exists<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            exists_impl(&inner, &name).await.map_err(backend_err_to_py)
        })
    }

    pub fn pid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            pid_impl(&inner, &name).await.map_err(backend_err_to_py)
        })
    }
}
```

## Step 2: Register in `modules/mod.rs`

Add `pub mod process;` to `src/modules/mod.rs`.

## Step 3: Add constructors to Host and AsyncHost

In `src/host.rs`, add methods to both `Host` and `AsyncHost`:

```rust
// In Host #[pymethods]
pub fn process(&self, name: &str) -> Process {
    Process {
        inner: self.inner.clone(),
        name: name.to_owned(),
    }
}

// In AsyncHost #[pymethods]
pub fn process(&self, name: &str) -> AsyncProcess {
    AsyncProcess {
        inner: self.inner.clone(),
        name: name.to_owned(),
    }
}
```

## Step 4: Register pyclasses in `lib.rs`

```rust
m.add_class::<modules::process::Process>()?;
m.add_class::<modules::process::AsyncProcess>()?;
```

## Step 5: Re-export in Python

Add to `python/oxi_nixinfra/__init__.py`:

```python
from oxi_nixinfra._oxi_nixinfra import Process, AsyncProcess
```

## Step 6: Write tests

Add a Rust unit test using `MockBackend` and a Python integration test.
See [Testing](testing.md) for the patterns.
