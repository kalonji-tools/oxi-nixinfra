use std::fmt;

pub mod local;
pub mod mock;
pub mod ssh;

use crate::command::RawOutput;

#[derive(Debug)]
pub enum BackendError {
    Connection(String),
    Execution(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(msg) => write!(f, "connection error: {msg}"),
            Self::Execution(msg) => write!(f, "execution error: {msg}"),
        }
    }
}

impl std::error::Error for BackendError {}

#[async_trait::async_trait]
pub trait Backend: Send + Sync {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<RawOutput, BackendError>;
}
