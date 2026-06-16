#![cfg(test)]

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::command::RawOutput;

use super::{Backend, BackendError};

pub struct MockBackend {
    responses: Mutex<VecDeque<RawOutput>>,
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl MockBackend {
    pub fn new(responses: Vec<RawOutput>) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from(responses)),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl Backend for MockBackend {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<RawOutput, BackendError> {
        self.calls.lock().unwrap().push((
            program.to_owned(),
            args.iter().map(|s| s.to_string()).collect(),
        ));
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| BackendError::Execution("no more mock responses".into()))
    }
}

mod tests {
    use super::*;

    fn raw(rc: i32, stdout: &str) -> RawOutput {
        RawOutput {
            rc,
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_mock_returns_fifo() {
        let backend = MockBackend::new(vec![raw(0, "first"), raw(1, "second")]);

        let r1 = backend.execute("cmd", &[]).await.unwrap();
        assert_eq!(r1.rc, 0);
        assert_eq!(r1.stdout, b"first");

        let r2 = backend.execute("cmd", &[]).await.unwrap();
        assert_eq!(r2.rc, 1);
        assert_eq!(r2.stdout, b"second");
    }

    #[tokio::test]
    async fn test_mock_records_calls() {
        let backend = MockBackend::new(vec![raw(0, "")]);
        backend.execute("nix", &["build", "--json"]).await.unwrap();

        let calls = backend.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "nix");
        assert_eq!(calls[0].1, vec!["build", "--json"]);
    }

    #[tokio::test]
    async fn test_mock_exhausted() {
        let backend = MockBackend::new(vec![]);
        let err = backend.execute("cmd", &[]).await.unwrap_err();
        assert!(matches!(err, BackendError::Execution(_)));
    }
}
