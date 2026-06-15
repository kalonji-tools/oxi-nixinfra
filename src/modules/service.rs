use std::collections::HashMap;
use std::sync::Arc;

use pyo3::prelude::*;

use crate::backend::BackendError;
use crate::host::HostInner;

// ---------------------------------------------------------------------------
// Layer 1 — Async core functions
// ---------------------------------------------------------------------------

pub async fn is_running_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let out = inner.execute("systemctl", &["is-active", name]).await?;
    Ok(out.rc == 0)
}

pub async fn is_enabled_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let out = inner.execute("systemctl", &["is-enabled", name]).await?;
    Ok(out.rc == 0)
}

pub async fn exists_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let out = inner
        .execute("systemctl", &["list-unit-files", "--no-pager"])
        .await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.lines().any(|line| line.starts_with(name)))
}

pub async fn is_masked_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let out = inner.execute("systemctl", &["is-enabled", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim() == "masked")
}

pub async fn is_valid_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let unit = if name.contains('.') {
        name.to_owned()
    } else {
        format!("{name}.service")
    };
    let out = inner.execute("systemd-analyze", &["verify", &unit]).await?;
    Ok(out.stdout.is_empty() && out.stderr.is_empty())
}

pub async fn properties_impl(
    inner: &HostInner,
    name: &str,
) -> Result<HashMap<String, String>, BackendError> {
    let out = inner.execute("systemctl", &["show", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let map = stdout
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    Ok(map)
}

// ---------------------------------------------------------------------------
// Layer 2 — Sync wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct Service {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) name: String,
}

#[pymethods]
impl Service {
    fn is_running(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_running_impl(&self.inner, &self.name))
    }

    fn is_enabled(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_enabled_impl(&self.inner, &self.name))
    }

    fn exists(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, exists_impl(&self.inner, &self.name))
    }

    fn is_masked(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_masked_impl(&self.inner, &self.name))
    }

    fn is_valid(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_valid_impl(&self.inner, &self.name))
    }

    fn properties(&self) -> PyResult<HashMap<String, String>> {
        crate::helpers::wrap_sync(&self.inner, properties_impl(&self.inner, &self.name))
    }

    fn __repr__(&self) -> String {
        format!("<Service {}>", self.name)
    }
}

// ---------------------------------------------------------------------------
// Layer 3 — Async wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct AsyncService {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) name: String,
}

#[pymethods]
impl AsyncService {
    fn is_running<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_running_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_enabled<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_enabled_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn exists<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            exists_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_masked<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_masked_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_valid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_valid_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            properties_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn __repr__(&self) -> String {
        format!("<AsyncService {}>", self.name)
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
    fn test_is_running_active() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"active\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(inner
            .runtime
            .block_on(is_running_impl(&inner, "nix-daemon"))
            .unwrap());
    }

    #[test]
    fn test_is_running_inactive() {
        let inner = make_inner(vec![RawOutput {
            rc: 3,
            stdout: b"inactive\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(!inner
            .runtime
            .block_on(is_running_impl(&inner, "nix-daemon"))
            .unwrap());
    }

    #[test]
    fn test_is_enabled_true() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"enabled\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(inner
            .runtime
            .block_on(is_enabled_impl(&inner, "nix-daemon"))
            .unwrap());
    }

    #[test]
    fn test_is_enabled_false() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: b"disabled\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(!inner
            .runtime
            .block_on(is_enabled_impl(&inner, "sshd"))
            .unwrap());
    }

    #[test]
    fn test_exists_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"nix-daemon.service  enabled  enabled\nsshd.service  enabled  enabled\n"
                .to_vec(),
            stderr: vec![],
        }]);
        assert!(inner
            .runtime
            .block_on(exists_impl(&inner, "nix-daemon"))
            .unwrap());
    }

    #[test]
    fn test_exists_not_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"sshd.service  enabled  enabled\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(!inner
            .runtime
            .block_on(exists_impl(&inner, "nix-daemon"))
            .unwrap());
    }

    #[test]
    fn test_is_masked() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: b"masked\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(inner
            .runtime
            .block_on(is_masked_impl(&inner, "masked-svc"))
            .unwrap());
    }

    #[test]
    fn test_properties_parse() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"Type=simple\nExecStart=/bin/foo\nDescription=\nActiveState=active\n"
                .to_vec(),
            stderr: vec![],
        }]);
        let props = inner
            .runtime
            .block_on(properties_impl(&inner, "test"))
            .unwrap();
        assert_eq!(props.get("Type").unwrap(), "simple");
        assert_eq!(props.get("ExecStart").unwrap(), "/bin/foo");
        assert_eq!(props.get("ActiveState").unwrap(), "active");
        assert!(!props.contains_key("Description"));
    }
}
