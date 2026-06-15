use crate::command::RawOutput;

use super::{Backend, BackendError};

pub struct LocalBackend;

#[async_trait::async_trait]
impl Backend for LocalBackend {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<RawOutput, BackendError> {
        let output = tokio::process::Command::new(program)
            .args(args)
            .output()
            .await
            .map_err(|e| BackendError::Execution(e.to_string()))?;
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

    #[tokio::test]
    async fn test_local_echo() {
        let backend = LocalBackend;
        let result = backend.execute("echo", &["hello"]).await.unwrap();
        assert_eq!(result.rc, 0);
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn test_local_nonzero_exit() {
        let backend = LocalBackend;
        let result = backend.execute("false", &[]).await.unwrap();
        assert_ne!(result.rc, 0);
    }

    #[tokio::test]
    async fn test_local_command_not_found() {
        let backend = LocalBackend;
        let result = backend
            .execute("surely-this-command-does-not-exist-xyz", &[])
            .await;
        assert!(result.is_err());
    }
}
