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

pub async fn is_managed_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let path = format!("/etc/systemd/system/{name}.service");
    let out = inner.execute("readlink", &["-f", &path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resolved = stdout.trim();
    Ok(out.rc == 0 && resolved.starts_with("/nix/store/"))
}

pub async fn store_path_impl(
    inner: &HostInner,
    name: &str,
) -> Result<Option<String>, BackendError> {
    let path = format!("/etc/systemd/system/{name}.service");
    let out = inner.execute("readlink", &["-f", &path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resolved = stdout.trim();
    if out.rc == 0 && resolved.starts_with("/nix/store/") {
        Ok(Some(resolved.to_owned()))
    } else {
        Ok(None)
    }
}

pub async fn enablement_status_impl(inner: &HostInner, name: &str) -> Result<String, BackendError> {
    let out = inner
        .execute(
            "systemctl",
            &["show", name, "-p", "UnitFileState", "--value"],
        )
        .await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_owned())
}

pub async fn exists_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let out = inner
        .execute("systemctl", &["list-unit-files", "--no-pager"])
        .await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.lines().any(|line| line.starts_with(name)))
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

    fn is_managed(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_managed_impl(&self.inner, &self.name))
    }

    fn enablement_status(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, enablement_status_impl(&self.inner, &self.name))
    }

    fn store_path(&self) -> PyResult<Option<String>> {
        crate::helpers::wrap_sync(&self.inner, store_path_impl(&self.inner, &self.name))
    }

    fn exists(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, exists_impl(&self.inner, &self.name))
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

    fn is_managed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_managed_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn enablement_status<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            enablement_status_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn store_path<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            store_path_impl(&inner, &name)
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
        assert!(
            inner
                .runtime
                .block_on(is_running_impl(&inner, "nix-daemon"))
                .unwrap()
        );
    }

    #[test]
    fn test_is_running_inactive() {
        let inner = make_inner(vec![RawOutput {
            rc: 3,
            stdout: b"inactive\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(
            !inner
                .runtime
                .block_on(is_running_impl(&inner, "nix-daemon"))
                .unwrap()
        );
    }

    #[test]
    fn test_is_managed_nix_store_path() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/nix/store/abc123-nix-2.31.4/lib/systemd/system/nix-daemon.service\n"
                .to_vec(),
            stderr: vec![],
        }]);
        assert!(
            inner
                .runtime
                .block_on(is_managed_impl(&inner, "nix-daemon"))
                .unwrap()
        );
    }

    #[test]
    fn test_is_managed_dev_null() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/dev/null\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(
            !inner
                .runtime
                .block_on(is_managed_impl(&inner, "console-getty"))
                .unwrap()
        );
    }

    #[test]
    fn test_is_managed_not_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: b"".to_vec(),
            stderr: b"readlink: /etc/systemd/system/sshd.service: No such file or directory\n"
                .to_vec(),
        }]);
        assert!(
            !inner
                .runtime
                .block_on(is_managed_impl(&inner, "sshd"))
                .unwrap()
        );
    }

    #[test]
    fn test_store_path_returns_path() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/nix/store/abc123-nix-2.31.4/lib/systemd/system/nix-daemon.service\n"
                .to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(store_path_impl(&inner, "nix-daemon"))
                .unwrap(),
            Some("/nix/store/abc123-nix-2.31.4/lib/systemd/system/nix-daemon.service".to_owned())
        );
    }

    #[test]
    fn test_store_path_returns_none_for_masked() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/dev/null\n".to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(store_path_impl(&inner, "console-getty"))
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_store_path_returns_none_for_not_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: b"".to_vec(),
            stderr: b"readlink: No such file or directory\n".to_vec(),
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(store_path_impl(&inner, "sshd"))
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_enablement_status_linked() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"linked\n".to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(enablement_status_impl(&inner, "nix-daemon"))
                .unwrap(),
            "linked"
        );
    }

    #[test]
    fn test_enablement_status_enabled() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"enabled\n".to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(enablement_status_impl(&inner, "avahi-daemon"))
                .unwrap(),
            "enabled"
        );
    }

    #[test]
    fn test_enablement_status_masked() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"masked\n".to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(enablement_status_impl(&inner, "console-getty"))
                .unwrap(),
            "masked"
        );
    }

    #[test]
    fn test_exists_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"nix-daemon.service  enabled  enabled\nsshd.service  enabled  enabled\n"
                .to_vec(),
            stderr: vec![],
        }]);
        assert!(
            inner
                .runtime
                .block_on(exists_impl(&inner, "nix-daemon"))
                .unwrap()
        );
    }

    #[test]
    fn test_exists_not_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"sshd.service  enabled  enabled\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(
            !inner
                .runtime
                .block_on(exists_impl(&inner, "nix-daemon"))
                .unwrap()
        );
    }

    #[test]
    fn test_properties_parse() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"Type=simple\nExecStart=/bin/foo\nDescription=\nActiveState=active\n".to_vec(),
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
