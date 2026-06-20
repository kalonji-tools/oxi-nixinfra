use std::sync::Arc;

use crate::command::RawOutput;

use super::{Backend, BackendError};

pub struct SshBackend {
    session: Arc<openssh::Session>,
}

impl SshBackend {
    pub async fn connect(
        host: &str,
        user: Option<&str>,
        port: Option<u16>,
        ssh_config: Option<&str>,
    ) -> Result<Self, BackendError> {
        let mut builder = openssh::SessionBuilder::default();
        if let Some(u) = user {
            builder.user(u.to_owned());
        }
        if let Some(p) = port {
            builder.port(p);
        }
        if let Some(config) = ssh_config {
            builder.config_file(config);
        }
        builder.known_hosts_check(openssh::KnownHosts::Accept);
        let session = builder
            .connect(host)
            .await
            .map_err(|e| BackendError::Connection(format!("SSH connect to {host} failed: {e}")))?;
        Ok(Self {
            session: Arc::new(session),
        })
    }
}

#[async_trait::async_trait]
impl Backend for SshBackend {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<RawOutput, BackendError> {
        let output = Arc::clone(&self.session)
            .arc_command(program)
            .args(args)
            .output()
            .await
            .map_err(|e| match e {
                openssh::Error::Disconnected => {
                    BackendError::Connection("SSH session disconnected".into())
                }
                other => BackendError::Execution(format!("SSH command failed: {other}")),
            })?;
        Ok(RawOutput {
            rc: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_error_display() {
        let err = BackendError::Connection("SSH connect to example.com failed: timeout".into());
        let msg = err.to_string();
        assert!(
            msg.contains("connection error"),
            "Connection variant should display as 'connection error: ...', got: {msg}"
        );
        assert!(
            msg.contains("example.com"),
            "error message should preserve the host, got: {msg}"
        );
    }

    #[test]
    fn execution_error_display() {
        let err = BackendError::Execution("SSH command failed: permission denied".into());
        let msg = err.to_string();
        assert!(
            msg.contains("execution error"),
            "Execution variant should display as 'execution error: ...', got: {msg}"
        );
        assert!(
            msg.contains("permission denied"),
            "error message should preserve the cause, got: {msg}"
        );
    }

    #[test]
    fn backend_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(BackendError::Connection("test".into()));
        assert!(
            err.to_string().contains("test"),
            "BackendError should implement std::error::Error Display"
        );
    }

    #[test]
    fn backend_error_debug_format() {
        let err = BackendError::Connection("debug test".into());
        let debug = format!("{err:?}");
        assert!(
            debug.contains("Connection"),
            "Debug format should include variant name, got: {debug}"
        );
    }

    #[test]
    fn ssh_backend_connect_string_format() {
        let host = "192.168.1.1";
        let err_msg = format!("SSH connect to {host} failed: connection refused");
        let err = BackendError::Connection(err_msg);
        assert!(
            err.to_string().contains(host),
            "connect error should include the target host"
        );
    }

    #[test]
    fn disconnected_error_message() {
        let err = BackendError::Connection("SSH session disconnected".into());
        assert_eq!(
            err.to_string(),
            "connection error: SSH session disconnected",
            "disconnection should produce a specific, recognizable message"
        );
    }
}
