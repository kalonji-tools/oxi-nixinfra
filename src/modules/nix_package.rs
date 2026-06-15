use std::sync::Arc;

use pyo3::prelude::*;

use crate::backend::BackendError;
use crate::host::HostInner;

// ---------------------------------------------------------------------------
// Layer 1 — Async core functions
// ---------------------------------------------------------------------------

async fn find_store_path(
    inner: &HostInner,
    name: &str,
) -> Result<Option<String>, BackendError> {
    let out = inner
        .execute("nix-store", &["-q", "--references", "/run/current-system/sw"])
        .await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout
        .lines()
        .find(|line| line.contains(name))
        .map(|l| l.to_owned()))
}

pub fn parse_version_from_store_path(store_path: &str, name: &str) -> Option<String> {
    let basename = store_path.rsplit('/').next()?;
    let after_hash = basename.get(33..)?;
    let name_pos = after_hash.find(name)?;
    let after_name = &after_hash[name_pos + name.len()..];
    after_name.strip_prefix('-').map(|v| v.to_owned())
}

pub async fn is_installed_impl(
    inner: &HostInner,
    name: &str,
) -> Result<bool, BackendError> {
    Ok(find_store_path(inner, name).await?.is_some())
}

pub async fn version_impl(
    inner: &HostInner,
    name: &str,
) -> Result<Option<String>, BackendError> {
    let path = find_store_path(inner, name).await?;
    Ok(path.and_then(|p| parse_version_from_store_path(&p, name)))
}

pub async fn store_path_impl(
    inner: &HostInner,
    name: &str,
) -> Result<Option<String>, BackendError> {
    find_store_path(inner, name).await
}

// ---------------------------------------------------------------------------
// Layer 2 — Sync wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct NixPackage {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) name: String,
}

#[pymethods]
impl NixPackage {
    fn is_installed(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_installed_impl(&self.inner, &self.name))
    }

    fn version(&self) -> PyResult<Option<String>> {
        crate::helpers::wrap_sync(&self.inner, version_impl(&self.inner, &self.name))
    }

    fn store_path(&self) -> PyResult<Option<String>> {
        crate::helpers::wrap_sync(&self.inner, store_path_impl(&self.inner, &self.name))
    }

    fn __repr__(&self) -> String {
        format!("<NixPackage {}>", self.name)
    }
}

// ---------------------------------------------------------------------------
// Layer 3 — Async wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct AsyncNixPackage {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) name: String,
}

#[pymethods]
impl AsyncNixPackage {
    fn is_installed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_installed_impl(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn version<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            version_impl(&inner, &name)
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

    fn __repr__(&self) -> String {
        format!("<AsyncNixPackage {}>", self.name)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_simple() {
        let path = "/nix/store/abc123abc123abc123abc123abc123ab-git-2.44.0";
        assert_eq!(
            parse_version_from_store_path(path, "git"),
            Some("2.44.0".to_owned())
        );
    }

    #[test]
    fn test_parse_version_hyphenated_name() {
        let path = "/nix/store/abc123abc123abc123abc123abc123ab-nix-output-monitor-2.1.2";
        assert_eq!(
            parse_version_from_store_path(path, "nix-output-monitor"),
            Some("2.1.2".to_owned())
        );
    }

    #[test]
    fn test_parse_version_no_match() {
        let path = "/nix/store/abc123abc123abc123abc123abc123ab-git-2.44.0";
        assert_eq!(parse_version_from_store_path(path, "vim"), None);
    }

    #[test]
    fn test_parse_version_no_version_suffix() {
        let path = "/nix/store/abc123abc123abc123abc123abc123ab-env-manifest.nix";
        assert_eq!(
            parse_version_from_store_path(path, "env-manifest.nix"),
            None
        );
    }

    #[test]
    fn test_is_installed_found() {
        use crate::backend::mock::MockBackend;
        use crate::command::RawOutput;

        let inner = HostInner {
            backend: Box::new(MockBackend::new(vec![RawOutput {
                rc: 0,
                stdout: b"/nix/store/abc123abc123abc123abc123abc123ab-git-2.44.0\n\
                          /nix/store/def456def456def456def456def456de-vim-9.1\n"
                    .to_vec(),
                stderr: vec![],
            }])),
            runtime: tokio::runtime::Runtime::new().unwrap(),
            connection_string: "mock://".to_owned(),
        };
        assert!(inner
            .runtime
            .block_on(is_installed_impl(&inner, "git"))
            .unwrap());
    }

    #[test]
    fn test_is_installed_not_found() {
        use crate::backend::mock::MockBackend;
        use crate::command::RawOutput;

        let inner = HostInner {
            backend: Box::new(MockBackend::new(vec![RawOutput {
                rc: 0,
                stdout: b"/nix/store/abc123abc123abc123abc123abc123ab-git-2.44.0\n".to_vec(),
                stderr: vec![],
            }])),
            runtime: tokio::runtime::Runtime::new().unwrap(),
            connection_string: "mock://".to_owned(),
        };
        assert!(!inner
            .runtime
            .block_on(is_installed_impl(&inner, "emacs"))
            .unwrap());
    }
}
