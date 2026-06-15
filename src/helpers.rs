use pyo3::exceptions::{PyConnectionError, PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::backend::BackendError;
use crate::host::HostInner;

pub fn wrap_sync<T>(
    inner: &HostInner,
    fut: impl std::future::Future<Output = Result<T, BackendError>>,
) -> PyResult<T> {
    inner.runtime.block_on(fut).map_err(backend_err_to_py)
}

pub fn backend_err_to_py(e: BackendError) -> PyErr {
    match e {
        BackendError::Connection(msg) => PyConnectionError::new_err(msg),
        BackendError::Execution(msg) => PyOSError::new_err(msg),
    }
}

pub fn extract_args(args: &Bound<'_, PyTuple>) -> PyResult<Vec<String>> {
    if args.is_empty() {
        return Err(PyValueError::new_err(
            "run() requires at least one argument (the program name)",
        ));
    }
    args.iter()
        .map(|a| a.extract::<String>())
        .collect::<PyResult<Vec<String>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error_maps_to_py_connection_error() {
        let err = backend_err_to_py(BackendError::Connection("host unreachable".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<PyConnectionError>(py));
            assert_eq!(err.value(py).to_string(), "host unreachable");
        });
    }

    #[test]
    fn test_execution_error_maps_to_py_os_error() {
        let err = backend_err_to_py(BackendError::Execution("command failed".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<PyOSError>(py));
            assert_eq!(err.value(py).to_string(), "command failed");
        });
    }
}
