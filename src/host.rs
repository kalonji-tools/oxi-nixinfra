use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::backend::local::LocalBackend;
use crate::backend::ssh::SshBackend;
use crate::backend::{Backend, BackendError};
use crate::command::CommandResult;
use crate::helpers::{backend_err_to_py, extract_args, wrap_sync};
use crate::parse::CommandOutput;

// ---------------------------------------------------------------------------
// Connection string parsing
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum Scheme {
    Local,
    Ssh {
        host: String,
        user: Option<String>,
        port: Option<u16>,
    },
}

pub fn parse_scheme(conn_str: &str) -> Result<Scheme, String> {
    if conn_str == "local://" || conn_str.is_empty() {
        return Ok(Scheme::Local);
    }
    if let Some(rest) = conn_str.strip_prefix("ssh://") {
        let (userinfo, hostport) = match rest.rsplit_once('@') {
            Some((u, h)) => (Some(u.to_owned()), h),
            None => (None, rest),
        };
        let (host, port) = match hostport.rsplit_once(':') {
            Some((h, p)) => {
                let port = p.parse::<u16>().map_err(|_| format!("invalid port: {p}"))?;
                (h.to_owned(), Some(port))
            }
            None => (hostport.to_owned(), None),
        };
        if host.is_empty() {
            return Err("empty hostname".into());
        }
        return Ok(Scheme::Ssh {
            host,
            user: userinfo,
            port,
        });
    }
    Err(format!("unknown scheme in connection string: {conn_str}"))
}

// ---------------------------------------------------------------------------
// HostInner — shared state behind Arc
// ---------------------------------------------------------------------------

pub struct HostInner {
    pub backend: Box<dyn Backend>,
    pub runtime: tokio::runtime::Runtime,
    pub connection_string: String,
}

impl HostInner {
    pub async fn execute(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<CommandOutput, BackendError> {
        let raw = self.backend.execute(program, args).await?;
        Ok(CommandOutput::from_raw(raw))
    }
}

// ---------------------------------------------------------------------------
// Global host cache
// ---------------------------------------------------------------------------

static HOST_CACHE: OnceLock<Mutex<HashMap<String, Arc<HostInner>>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, Arc<HostInner>>> {
    HOST_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_or_create_host(conn_str: &str, _ssh_config: Option<&str>) -> PyResult<Arc<HostInner>> {
    let mut map = cache().lock().unwrap();
    if let Some(inner) = map.get(conn_str) {
        return Ok(Arc::clone(inner));
    }

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| PyValueError::new_err(format!("failed to create tokio runtime: {e}")))?;

    let scheme =
        parse_scheme(conn_str).map_err(|e| PyValueError::new_err(format!("bad URI: {e}")))?;

    let backend: Box<dyn Backend> = match scheme {
        Scheme::Local => Box::new(LocalBackend),
        Scheme::Ssh { host, user, port } => {
            let b = runtime
                .block_on(SshBackend::connect(&host, user.as_deref(), port))
                .map_err(backend_err_to_py)?;
            Box::new(b)
        }
    };

    let inner = Arc::new(HostInner {
        backend,
        runtime,
        connection_string: conn_str.to_owned(),
    });
    map.insert(conn_str.to_owned(), Arc::clone(&inner));
    Ok(inner)
}

// ---------------------------------------------------------------------------
// Host (synchronous pyclass)
// ---------------------------------------------------------------------------

#[pyclass]
pub struct Host {
    inner: Arc<HostInner>,
}

#[pymethods]
impl Host {
    #[staticmethod]
    #[pyo3(signature = (host_str, ssh_config = None))]
    fn _from_config(host_str: &str, ssh_config: Option<&str>) -> PyResult<Self> {
        let inner = get_or_create_host(host_str, ssh_config)?;
        Ok(Self { inner })
    }

    #[pyo3(signature = (*args))]
    fn run(&self, args: &Bound<'_, PyTuple>) -> PyResult<CommandResult> {
        let parts = extract_args(args)?;
        let program = &parts[0];
        let str_args: Vec<&str> = parts[1..].iter().map(std::string::String::as_str).collect();
        let command_display = parts.join(" ");
        let out = wrap_sync(&self.inner, self.inner.execute(program, &str_args))?;
        Ok(CommandResult::from_output(out, command_display))
    }

    #[getter]
    fn a(&self) -> AsyncHost {
        AsyncHost {
            inner: Arc::clone(&self.inner),
        }
    }

    fn __repr__(&self) -> String {
        format!("<Host {}>", self.inner.connection_string)
    }
}

// ---------------------------------------------------------------------------
// AsyncHost (async pyclass)
// ---------------------------------------------------------------------------

#[pyclass]
pub struct AsyncHost {
    inner: Arc<HostInner>,
}

#[pymethods]
impl AsyncHost {
    #[pyo3(signature = (*args))]
    fn run<'py>(&self, py: Python<'py>, args: &Bound<'py, PyTuple>) -> PyResult<Bound<'py, PyAny>> {
        let parts = extract_args(args)?;
        let inner = Arc::clone(&self.inner);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let program = &parts[0];
            let str_args: Vec<&str> = parts[1..].iter().map(std::string::String::as_str).collect();
            let command_display = parts.join(" ");
            let out = inner
                .execute(program, &str_args)
                .await
                .map_err(backend_err_to_py)?;
            Ok(CommandResult::from_output(out, command_display))
        })
    }

    fn __repr__(&self) -> String {
        format!("<AsyncHost {}>", self.inner.connection_string)
    }
}

oxi_nixinfra_macros::register_modules! {
    Host, AsyncHost;
    service(name: &str) -> service::Service,
    file(path: &str) -> file::File,
    user(name: Option<&str> = None) -> user::User,
    system_info() -> system_info::SystemInfo,
    nix_package(name: &str) -> nix_package::NixPackage,
    process() -> process::Process,
    socket(spec: &str) -> socket::Socket,
    mountpoint(path: &str) -> mountpoint::MountPoint,
    sysctl(key: &str) -> sysctl::Sysctl,
    environment() -> environment::Environment,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local() {
        assert_eq!(parse_scheme("local://").unwrap(), Scheme::Local);
    }

    #[test]
    fn test_parse_empty_string() {
        assert_eq!(parse_scheme("").unwrap(), Scheme::Local);
    }

    #[test]
    fn test_parse_ssh_host_only() {
        assert_eq!(
            parse_scheme("ssh://myhost").unwrap(),
            Scheme::Ssh {
                host: "myhost".into(),
                user: None,
                port: None,
            }
        );
    }

    #[test]
    fn test_parse_ssh_user_host() {
        assert_eq!(
            parse_scheme("ssh://me@myhost").unwrap(),
            Scheme::Ssh {
                host: "myhost".into(),
                user: Some("me".into()),
                port: None,
            }
        );
    }

    #[test]
    fn test_parse_ssh_user_host_port() {
        assert_eq!(
            parse_scheme("ssh://me@myhost:2222").unwrap(),
            Scheme::Ssh {
                host: "myhost".into(),
                user: Some("me".into()),
                port: Some(2222),
            }
        );
    }

    #[test]
    fn test_parse_ssh_host_port() {
        assert_eq!(
            parse_scheme("ssh://myhost:22").unwrap(),
            Scheme::Ssh {
                host: "myhost".into(),
                user: None,
                port: Some(22),
            }
        );
    }

    #[test]
    fn test_parse_unknown_scheme() {
        let err = parse_scheme("docker://foo").unwrap_err();
        assert!(err.contains("unknown scheme"));
    }

    #[test]
    fn test_parse_ssh_empty_host() {
        let err = parse_scheme("ssh://:22").unwrap_err();
        assert!(err.contains("empty hostname"));
    }

    #[test]
    fn test_parse_ssh_invalid_port() {
        let err = parse_scheme("ssh://host:notaport").unwrap_err();
        assert!(err.contains("invalid port"));
    }
}
