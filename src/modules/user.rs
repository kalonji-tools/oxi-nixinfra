use std::sync::Arc;

use pyo3::prelude::*;

use crate::backend::BackendError;
use crate::host::HostInner;

// ---------------------------------------------------------------------------
// Layer 1 — Async core functions
// ---------------------------------------------------------------------------

pub async fn exists_impl(inner: &HostInner, name: &str) -> Result<bool, BackendError> {
    let out = inner.execute("id", &[name]).await?;
    Ok(out.rc == 0)
}

pub async fn name_impl(inner: &HostInner) -> Result<String, BackendError> {
    let out = inner.execute("id", &["-nu"]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_owned())
}

pub async fn uid_impl(inner: &HostInner, name: &str) -> Result<i32, BackendError> {
    let out = inner.execute("id", &["-u", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .trim()
        .parse::<i32>()
        .map_err(|e| BackendError::Execution(format!("failed to parse uid: {e}")))
}

pub async fn gid_impl(inner: &HostInner, name: &str) -> Result<i32, BackendError> {
    let out = inner.execute("id", &["-g", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .trim()
        .parse::<i32>()
        .map_err(|e| BackendError::Execution(format!("failed to parse gid: {e}")))
}

pub async fn group_impl(inner: &HostInner, name: &str) -> Result<String, BackendError> {
    let out = inner.execute("id", &["-ng", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_owned())
}

pub async fn groups_impl(inner: &HostInner, name: &str) -> Result<Vec<String>, BackendError> {
    let out = inner.execute("id", &["-nG", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.split_whitespace().map(|s| s.to_owned()).collect())
}

async fn getent_field(
    inner: &HostInner,
    name: &str,
    index: usize,
) -> Result<String, BackendError> {
    let out = inner.execute("getent", &["passwd", name]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.trim();
    line.split(':')
        .nth(index)
        .map(|s| s.to_owned())
        .ok_or_else(|| {
            BackendError::Execution(format!(
                "failed to parse getent field {index} for user: {name}"
            ))
        })
}

pub async fn home_impl(inner: &HostInner, name: &str) -> Result<String, BackendError> {
    getent_field(inner, name, 5).await
}

pub async fn shell_impl(inner: &HostInner, name: &str) -> Result<String, BackendError> {
    getent_field(inner, name, 6).await
}

/// Resolve the effective user name: use stored name if present, otherwise run `id -nu`.
async fn resolve_name(inner: &HostInner, name: &Option<String>) -> Result<String, BackendError> {
    match name {
        Some(n) => Ok(n.clone()),
        None => name_impl(inner).await,
    }
}

// ---------------------------------------------------------------------------
// Layer 2 — Sync wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct User {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) name: Option<String>,
}

#[pymethods]
impl User {
    fn exists(&self) -> PyResult<bool> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, exists_impl(inner, &name))
    }

    fn name(&self) -> PyResult<String> {
        match &self.name {
            Some(n) => Ok(n.clone()),
            None => crate::helpers::wrap_sync(&self.inner, name_impl(&self.inner)),
        }
    }

    fn uid(&self) -> PyResult<i32> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, uid_impl(inner, &name))
    }

    fn gid(&self) -> PyResult<i32> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, gid_impl(inner, &name))
    }

    fn group(&self) -> PyResult<String> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, group_impl(inner, &name))
    }

    fn groups(&self) -> PyResult<Vec<String>> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, groups_impl(inner, &name))
    }

    fn home(&self) -> PyResult<String> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, home_impl(inner, &name))
    }

    fn shell(&self) -> PyResult<String> {
        let inner = &self.inner;
        let name = crate::helpers::wrap_sync(inner, resolve_name(inner, &self.name))?;
        crate::helpers::wrap_sync(inner, shell_impl(inner, &name))
    }

    fn __repr__(&self) -> String {
        match &self.name {
            Some(n) => format!("<User {n}>"),
            None => "<User (current)>".to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 3 — Async wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct AsyncUser {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) name: Option<String>,
}

#[pymethods]
impl AsyncUser {
    fn exists<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            exists_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn name<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            match name {
                Some(n) => Ok(n),
                None => name_impl(&inner)
                    .await
                    .map_err(crate::helpers::backend_err_to_py),
            }
        })
    }

    fn uid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            uid_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn gid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            gid_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn group<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            group_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn groups<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            groups_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn home<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            home_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn shell<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let name = self.name.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resolved = resolve_name(&inner, &name)
                .await
                .map_err(crate::helpers::backend_err_to_py)?;
            shell_impl(&inner, &resolved)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn __repr__(&self) -> String {
        match &self.name {
            Some(n) => format!("<AsyncUser {n}>"),
            None => "<AsyncUser (current)>".to_owned(),
        }
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
    fn test_getent_field_home() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"testuser:x:1000:1000:Test User:/home/testuser:/bin/bash\n".to_vec(),
            stderr: vec![],
        }]);
        let home = inner
            .runtime
            .block_on(getent_field(&inner, "testuser", 5))
            .unwrap();
        assert_eq!(home, "/home/testuser");
    }

    #[test]
    fn test_getent_field_shell() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"testuser:x:1000:1000:Test User:/home/testuser:/bin/zsh\n".to_vec(),
            stderr: vec![],
        }]);
        let shell = inner
            .runtime
            .block_on(getent_field(&inner, "testuser", 6))
            .unwrap();
        assert_eq!(shell, "/bin/zsh");
    }

    #[test]
    fn test_groups_splitting() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"wheel docker users\n".to_vec(),
            stderr: vec![],
        }]);
        let groups = inner
            .runtime
            .block_on(groups_impl(&inner, "testuser"))
            .unwrap();
        assert_eq!(groups, vec!["wheel", "docker", "users"]);
    }

    #[test]
    fn test_uid_parse() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"1000\n".to_vec(),
            stderr: vec![],
        }]);
        let uid = inner
            .runtime
            .block_on(uid_impl(&inner, "testuser"))
            .unwrap();
        assert_eq!(uid, 1000);
    }
}
