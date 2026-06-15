use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use pyo3::prelude::*;

use crate::backend::BackendError;
use crate::host::HostInner;

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

pub fn parse_os_release(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            let value = value.trim_matches('"').trim_matches('\'').to_owned();
            Some((key.to_owned(), value))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Layer 1 — Async core function
// ---------------------------------------------------------------------------

pub async fn sysinfo_impl(inner: &HostInner) -> Result<HashMap<String, String>, BackendError> {
    let os_release = inner.execute("cat", &["/etc/os-release"]).await?;
    let uname_s = inner.execute("uname", &["-s"]).await?;
    let uname_m = inner.execute("uname", &["-m"]).await?;

    let mut info = parse_os_release(&String::from_utf8_lossy(&os_release.stdout));

    let sys_type = String::from_utf8_lossy(&uname_s.stdout)
        .trim()
        .to_lowercase();
    info.insert("type".to_owned(), sys_type);

    let arch = String::from_utf8_lossy(&uname_m.stdout).trim().to_owned();
    info.insert("arch".to_owned(), arch);

    Ok(info)
}

fn extract_field(info: &HashMap<String, String>, key: &str) -> Option<String> {
    info.get(key).filter(|v| !v.is_empty()).cloned()
}

// ---------------------------------------------------------------------------
// Layer 2 — Sync wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct SystemInfo {
    pub(crate) inner: Arc<HostInner>,
    cache: Mutex<Option<HashMap<String, String>>>,
}

impl SystemInfo {
    pub fn new(inner: Arc<HostInner>) -> Self {
        Self {
            inner,
            cache: Mutex::new(None),
        }
    }

    fn ensure_loaded(&self) -> PyResult<HashMap<String, String>> {
        let mut guard = self.cache.lock().unwrap();
        if let Some(ref cached) = *guard {
            return Ok(cached.clone());
        }
        let info = crate::helpers::wrap_sync(&self.inner, sysinfo_impl(&self.inner))?;
        *guard = Some(info.clone());
        Ok(info)
    }
}

#[pymethods]
impl SystemInfo {
    #[pyo3(name = "type")]
    fn type_(&self) -> PyResult<Option<String>> {
        Ok(extract_field(&self.ensure_loaded()?, "type"))
    }

    fn distribution(&self) -> PyResult<Option<String>> {
        Ok(extract_field(&self.ensure_loaded()?, "ID"))
    }

    fn release(&self) -> PyResult<Option<String>> {
        Ok(extract_field(&self.ensure_loaded()?, "VERSION_ID"))
    }

    fn codename(&self) -> PyResult<Option<String>> {
        Ok(extract_field(&self.ensure_loaded()?, "VERSION_CODENAME"))
    }

    fn arch(&self) -> PyResult<Option<String>> {
        Ok(extract_field(&self.ensure_loaded()?, "arch"))
    }

    fn __repr__(&self) -> String {
        "<SystemInfo>".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Layer 3 — Async wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct AsyncSystemInfo {
    pub(crate) inner: Arc<HostInner>,
}

#[pymethods]
impl AsyncSystemInfo {
    #[pyo3(name = "type")]
    fn type_<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let info = sysinfo_impl(&inner)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            Ok(extract_field(&info, "type"))
        })
    }

    fn distribution<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let info = sysinfo_impl(&inner)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            Ok(extract_field(&info, "ID"))
        })
    }

    fn release<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let info = sysinfo_impl(&inner)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            Ok(extract_field(&info, "VERSION_ID"))
        })
    }

    fn codename<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let info = sysinfo_impl(&inner)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            Ok(extract_field(&info, "VERSION_CODENAME"))
        })
    }

    fn arch<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let info = sysinfo_impl(&inner)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            Ok(extract_field(&info, "arch"))
        })
    }

    fn __repr__(&self) -> String {
        "<AsyncSystemInfo>".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_os_release_quoted() {
        let content = r#"NAME="NixOS"
ID=nixos
VERSION_ID="24.05"
VERSION_CODENAME=""
PRETTY_NAME="NixOS 24.05 (Uakari)"
"#;
        let info = parse_os_release(content);
        assert_eq!(info.get("NAME").unwrap(), "NixOS");
        assert_eq!(info.get("ID").unwrap(), "nixos");
        assert_eq!(info.get("VERSION_ID").unwrap(), "24.05");
        assert_eq!(info.get("VERSION_CODENAME").unwrap(), "");
    }

    #[test]
    fn test_parse_os_release_unquoted() {
        let content = "ID=nixos\nVERSION_ID=24.05\n";
        let info = parse_os_release(content);
        assert_eq!(info.get("ID").unwrap(), "nixos");
        assert_eq!(info.get("VERSION_ID").unwrap(), "24.05");
    }

    #[test]
    fn test_parse_os_release_single_quoted() {
        let content = "NAME='NixOS'\n";
        let info = parse_os_release(content);
        assert_eq!(info.get("NAME").unwrap(), "NixOS");
    }

    #[test]
    fn test_extract_field_empty_is_none() {
        let mut info = HashMap::new();
        info.insert("VERSION_CODENAME".to_owned(), String::new());
        assert!(extract_field(&info, "VERSION_CODENAME").is_none());
    }

    #[test]
    fn test_extract_field_missing_is_none() {
        let info = HashMap::new();
        assert!(extract_field(&info, "NONEXISTENT").is_none());
    }

    #[test]
    fn test_extract_field_present() {
        let mut info = HashMap::new();
        info.insert("ID".to_owned(), "nixos".to_owned());
        assert_eq!(extract_field(&info, "ID").unwrap(), "nixos");
    }
}
