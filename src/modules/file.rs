use std::sync::Arc;

use pyo3::prelude::*;

use crate::backend::BackendError;
use crate::host::HostInner;

// ---------------------------------------------------------------------------
// Layer 1 — Async core functions
// ---------------------------------------------------------------------------

pub async fn exists_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-e", path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_file_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-f", path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_directory_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-d", path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_symlink_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-L", path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_executable_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-x", path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_pipe_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-p", path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_socket_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("test", &["-S", path]).await?;
    Ok(out.rc == 0)
}

pub async fn linked_to_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("realpath", &[path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_owned())
}

pub async fn user_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("stat", &["-Lc", "%U", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_owned())
}

pub async fn uid_impl(inner: &HostInner, path: &str) -> Result<i32, BackendError> {
    let out = inner.execute("stat", &["-Lc", "%u", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .trim()
        .parse::<i32>()
        .map_err(|e| BackendError::Execution(format!("failed to parse uid: {e}")))
}

pub async fn group_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("stat", &["-Lc", "%G", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_owned())
}

pub async fn gid_impl(inner: &HostInner, path: &str) -> Result<i32, BackendError> {
    let out = inner.execute("stat", &["-Lc", "%g", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .trim()
        .parse::<i32>()
        .map_err(|e| BackendError::Execution(format!("failed to parse gid: {e}")))
}

pub async fn mode_impl(inner: &HostInner, path: &str) -> Result<i32, BackendError> {
    let out = inner.execute("stat", &["-Lc", "%a", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    i32::from_str_radix(stdout.trim(), 8)
        .map_err(|e| BackendError::Execution(format!("failed to parse mode: {e}")))
}

pub async fn size_impl(inner: &HostInner, path: &str) -> Result<u64, BackendError> {
    let out = inner.execute("stat", &["-Lc", "%s", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .trim()
        .parse::<u64>()
        .map_err(|e| BackendError::Execution(format!("failed to parse size: {e}")))
}

pub async fn content_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("cat", &["--", path]).await?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub async fn contains_impl(
    inner: &HostInner,
    path: &str,
    pattern: &str,
) -> Result<bool, BackendError> {
    let out = inner.execute("grep", &["-qs", "--", pattern, path]).await?;
    Ok(out.rc == 0)
}

pub async fn is_nix_managed_impl(inner: &HostInner, path: &str) -> Result<bool, BackendError> {
    let out = inner.execute("readlink", &["-f", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(out.rc == 0 && stdout.trim().starts_with("/nix/store/"))
}

pub async fn file_store_path_impl(
    inner: &HostInner,
    path: &str,
) -> Result<Option<String>, BackendError> {
    let out = inner.execute("readlink", &["-f", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resolved = stdout.trim();
    if out.rc == 0 && resolved.starts_with("/nix/store/") {
        Ok(Some(resolved.to_owned()))
    } else {
        Ok(None)
    }
}

pub async fn md5sum_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("md5sum", &[path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .split_whitespace()
        .next()
        .map(std::borrow::ToOwned::to_owned)
        .ok_or_else(|| BackendError::Execution(format!("failed to parse md5sum for: {path}")))
}

pub async fn sha256sum_impl(inner: &HostInner, path: &str) -> Result<String, BackendError> {
    let out = inner.execute("sha256sum", &[path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .split_whitespace()
        .next()
        .map(std::borrow::ToOwned::to_owned)
        .ok_or_else(|| BackendError::Execution(format!("failed to parse sha256sum for: {path}")))
}

pub async fn listdir_impl(inner: &HostInner, path: &str) -> Result<Vec<String>, BackendError> {
    let out = inner.execute("ls", &["-1", "-q", "--", path]).await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.lines().map(std::borrow::ToOwned::to_owned).collect())
}

// ---------------------------------------------------------------------------
// Layer 2 — Sync wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct File {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) path: String,
}

#[pymethods]
impl File {
    fn exists(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, exists_impl(&self.inner, &self.path))
    }

    fn is_file(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_file_impl(&self.inner, &self.path))
    }

    fn is_directory(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_directory_impl(&self.inner, &self.path))
    }

    fn is_symlink(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_symlink_impl(&self.inner, &self.path))
    }

    fn is_executable(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_executable_impl(&self.inner, &self.path))
    }

    fn is_pipe(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_pipe_impl(&self.inner, &self.path))
    }

    fn is_socket(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_socket_impl(&self.inner, &self.path))
    }

    fn linked_to(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, linked_to_impl(&self.inner, &self.path))
    }

    fn user(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, user_impl(&self.inner, &self.path))
    }

    fn uid(&self) -> PyResult<i32> {
        crate::helpers::wrap_sync(&self.inner, uid_impl(&self.inner, &self.path))
    }

    fn group(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, group_impl(&self.inner, &self.path))
    }

    fn gid(&self) -> PyResult<i32> {
        crate::helpers::wrap_sync(&self.inner, gid_impl(&self.inner, &self.path))
    }

    fn mode(&self) -> PyResult<i32> {
        crate::helpers::wrap_sync(&self.inner, mode_impl(&self.inner, &self.path))
    }

    fn size(&self) -> PyResult<u64> {
        crate::helpers::wrap_sync(&self.inner, size_impl(&self.inner, &self.path))
    }

    fn content(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, content_impl(&self.inner, &self.path))
    }

    fn contains(&self, pattern: &str) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, contains_impl(&self.inner, &self.path, pattern))
    }

    fn md5sum(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, md5sum_impl(&self.inner, &self.path))
    }

    fn sha256sum(&self) -> PyResult<String> {
        crate::helpers::wrap_sync(&self.inner, sha256sum_impl(&self.inner, &self.path))
    }

    fn listdir(&self) -> PyResult<Vec<String>> {
        crate::helpers::wrap_sync(&self.inner, listdir_impl(&self.inner, &self.path))
    }

    fn is_nix_managed(&self) -> PyResult<bool> {
        crate::helpers::wrap_sync(&self.inner, is_nix_managed_impl(&self.inner, &self.path))
    }

    fn store_path(&self) -> PyResult<Option<String>> {
        crate::helpers::wrap_sync(&self.inner, file_store_path_impl(&self.inner, &self.path))
    }

    fn __repr__(&self) -> String {
        format!("<File {}>", self.path)
    }
}

// ---------------------------------------------------------------------------
// Layer 3 — Async wrapper
// ---------------------------------------------------------------------------

#[pyclass(frozen)]
pub struct AsyncFile {
    pub(crate) inner: Arc<HostInner>,
    pub(crate) path: String,
}

#[pymethods]
impl AsyncFile {
    fn exists<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            exists_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_file<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_file_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_directory<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_directory_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_symlink<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_symlink_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_executable<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_executable_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_pipe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_pipe_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_socket<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_socket_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn linked_to<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            linked_to_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn user<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            user_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn uid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            uid_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn group<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            group_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn gid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            gid_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn mode<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            mode_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn size<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            size_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn content<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            content_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn contains<'py>(&self, py: Python<'py>, pattern: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        let pattern = pattern.to_owned();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            contains_impl(&inner, &path, &pattern)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn md5sum<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            md5sum_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn sha256sum<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            sha256sum_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn listdir<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            listdir_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn is_nix_managed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            is_nix_managed_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn store_path<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let path = self.path.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            file_store_path_impl(&inner, &path)
                .await
                .map_err(crate::helpers::backend_err_to_py)
        })
    }

    fn __repr__(&self) -> String {
        format!("<AsyncFile {}>", self.path)
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
    fn test_mode_octal_parse() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"644\n".to_vec(),
            stderr: vec![],
        }]);
        let mode = inner
            .runtime
            .block_on(mode_impl(&inner, "/etc/passwd"))
            .unwrap();
        assert_eq!(mode, 0o644);
    }

    #[test]
    fn test_md5sum_split_field() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"d41d8cd98f00b204e9800998ecf8427e  /dev/null\n".to_vec(),
            stderr: vec![],
        }]);
        let hash = inner
            .runtime
            .block_on(md5sum_impl(&inner, "/dev/null"))
            .unwrap();
        assert_eq!(hash, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_uid_parse_int() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"1000\n".to_vec(),
            stderr: vec![],
        }]);
        let uid = inner
            .runtime
            .block_on(uid_impl(&inner, "/home/user"))
            .unwrap();
        assert_eq!(uid, 1000);
    }

    #[test]
    fn test_contains_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: vec![],
            stderr: vec![],
        }]);
        assert!(
            inner
                .runtime
                .block_on(contains_impl(&inner, "/etc/hosts", "localhost"))
                .unwrap()
        );
    }

    #[test]
    fn test_contains_not_found() {
        let inner = make_inner(vec![RawOutput {
            rc: 1,
            stdout: vec![],
            stderr: vec![],
        }]);
        assert!(
            !inner
                .runtime
                .block_on(contains_impl(&inner, "/etc/hosts", "nonexistent"))
                .unwrap()
        );
    }

    #[test]
    fn test_is_nix_managed_true() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/nix/store/h0p1rl4srn0j3arkiahpwrv8fp8vpp8l-hosts\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(
            inner
                .runtime
                .block_on(is_nix_managed_impl(&inner, "/etc/hosts"))
                .unwrap()
        );
    }

    #[test]
    fn test_is_nix_managed_false() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/etc/resolv.conf\n".to_vec(),
            stderr: vec![],
        }]);
        assert!(
            !inner
                .runtime
                .block_on(is_nix_managed_impl(&inner, "/etc/resolv.conf"))
                .unwrap()
        );
    }

    #[test]
    fn test_file_store_path_some() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/nix/store/x1nwcwf197wb5d7infxr4il5l4gzpdp3-nix.conf\n".to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(file_store_path_impl(&inner, "/etc/nix/nix.conf"))
                .unwrap(),
            Some("/nix/store/x1nwcwf197wb5d7infxr4il5l4gzpdp3-nix.conf".to_owned())
        );
    }

    #[test]
    fn test_file_store_path_none() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"/etc/passwd\n".to_vec(),
            stderr: vec![],
        }]);
        assert_eq!(
            inner
                .runtime
                .block_on(file_store_path_impl(&inner, "/etc/passwd"))
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_listdir_parse() {
        let inner = make_inner(vec![RawOutput {
            rc: 0,
            stdout: b"file1.txt\nfile2.txt\ndir1\n".to_vec(),
            stderr: vec![],
        }]);
        let entries = inner
            .runtime
            .block_on(listdir_impl(&inner, "/tmp"))
            .unwrap();
        assert_eq!(entries, vec!["file1.txt", "file2.txt", "dir1"]);
    }
}
