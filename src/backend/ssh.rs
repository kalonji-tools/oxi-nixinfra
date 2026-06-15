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
    ) -> Result<Self, BackendError> {
        let mut builder = openssh::SessionBuilder::default();
        if let Some(u) = user {
            builder.user(u.to_owned());
        }
        if let Some(p) = port {
            builder.port(p);
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
