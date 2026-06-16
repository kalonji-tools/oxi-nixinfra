use std::sync::Arc;

use pyo3::prelude::*;

use crate::backend::BackendError;
use crate::host::HostInner;

// ---------------------------------------------------------------------------
// Layer 1 — Async core functions
// ---------------------------------------------------------------------------

pub async fn exists_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("nixos-option", &[path]).await?;
    Ok(out.rc == 0)
}

pub async fn value_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("nixos-option", &[path]).await?;
    if out.rc == 0 {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for (i, line) in stdout.lines().enumerate() {
            if line.starts_with("Value:") {
                return stdout
                    .lines()
                    .skip(i + 1)
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.trim().to_owned())
                    .ok_or_else(|| {
                        BackendError::Execution(format!("could not parse value for: {path}"))
                    });
            }
        }
    }
    Err(BackendError::Execution(format!(
        "nixos-option failed for '{}': {}",
        path,
        String::from_utf8_lossy(&out.stderr)
    )))
}

// ---------------------------------------------------------------------------
// Layer 2 — Sync wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct NixOption {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) path: String,
}

#[pymethods]
impl NixOption {
    fn exists(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, exists_impl(&self.inner, &self.path))
    }

    fn value(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, value_impl(&self.inner, &self.path))
    }

    fn __repr__(&self) -> String {
        format!("<NixOption {}>", self.path)
    }
}

// ---------------------------------------------------------------------------
// Layer 3 — Async wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct AsyncNixOption {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) path: String,
}

#[pymethods]
impl AsyncNixOption {
    fn exists<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            exists_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            value_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn __repr__(&self) -> String {
        format!("<AsyncNixOption {}>", self.path)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::mock::MockBackend;
    use crate::command::RawOutput;

    fn make_inner(responses: Vec<RawOutput>) -> HostInner {
        HostInner {
            backend: Box::new(MockBackend::new(responses)),
            runtime: tokio::runtime::Runtime::new().unwrap(),
            connection_string: "mock://".to_owned(),
        }
    }

    #[test]
    fn test_exists_true() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"Value:\n  true\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(
            inner
                .runtime
                .block_on(exists_impl(&inner, "services.openssh.enable"))
                .unwrap()
        );
    }

    #[test]
    fn test_exists_false() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: vec![],
            stderr: b"error: option not found\n".to_vec(),
        }]);
        assert!(
            !inner
                .runtime
                .block_on(exists_impl(&inner, "services.nonexistent"))
                .unwrap()
        );
    }

    #[test]
    fn test_value_parse() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"Value:\n  true\n\nDefault:\n  false\n".to_vec(),
            stderr: vec![],
        }]);
        let val = inner
            .runtime
            .block_on(value_impl(&inner, "services.openssh.enable"))
            .unwrap();
        assert_eq!(val, "true");
    }

    #[test]
    fn test_value_error_on_failure() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: vec![],
            stderr: b"error: option not found\n".to_vec(),
        }]);
        let err = inner
            .runtime
            .block_on(value_impl(&inner, "nonexistent.option"))
            .unwrap_err();
        assert!(err.to_string().contains("nixos-option failed"));
    }
}
