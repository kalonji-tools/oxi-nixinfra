use pyo3::prelude::*;

/// Raw output from a subprocess execution (internal, not exported to Python).
#[derive(Debug)]
pub struct RawOutput {
    pub rc: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Result of a command execution, exposed to Python.
#[pyclass(frozen, get_all)]
#[derive(Clone)]
pub struct CommandResult {
    pub rc: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[pymethods]
impl CommandResult {
    fn succeeded(&self) -> bool {
        self.rc == 0
    }

    fn failed(&self) -> bool {
        self.rc != 0
    }

    fn __repr__(&self) -> String {
        format!("<CommandResult rc={} command='{}'>", self.rc, self.command)
    }
}

impl CommandResult {
    pub fn from_raw(raw: RawOutput, command: String) -> Self {
        Self {
            rc: raw.rc,
            stdout: String::from_utf8_lossy(&raw.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&raw.stderr).into_owned(),
            command,
        }
    }

    #[cfg(test)]
    fn is_succeeded(&self) -> bool {
        self.rc == 0
    }

    #[cfg(test)]
    fn is_failed(&self) -> bool {
        self.rc != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_success() {
        let raw = RawOutput {
            rc: 0,
            stdout: b"hello\n".to_vec(),
            stderr: Vec::new(),
        };
        let result = CommandResult::from_raw(raw, "echo hello".into());
        assert_eq!(result.rc, 0);
        assert_eq!(result.stdout, "hello\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.command, "echo hello");
    }

    #[test]
    fn test_from_raw_failure() {
        let raw = RawOutput {
            rc: 1,
            stdout: Vec::new(),
            stderr: b"not found\n".to_vec(),
        };
        let result = CommandResult::from_raw(raw, "missing-cmd".into());
        assert_eq!(result.rc, 1);
        assert_eq!(result.stderr, "not found\n");
    }

    #[test]
    fn test_succeeded_and_failed() {
        let success = CommandResult::from_raw(
            RawOutput {
                rc: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
            "true".into(),
        );
        assert!(success.is_succeeded());
        assert!(!success.is_failed());

        let failure = CommandResult::from_raw(
            RawOutput {
                rc: 1,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
            "false".into(),
        );
        assert!(!failure.is_succeeded());
        assert!(failure.is_failed());
    }

    #[test]
    fn test_lossy_utf8() {
        let raw = RawOutput {
            rc: 0,
            stdout: vec![0xFF, 0xFE, b'h', b'i'],
            stderr: vec![0x80, 0x81],
        };
        let result = CommandResult::from_raw(raw, "binary-cmd".into());
        assert!(result.stdout.contains("hi"));
        assert!(result.stdout.contains('\u{FFFD}'));
        assert!(result.stderr.contains('\u{FFFD}'));
    }

    #[test]
    fn test_repr() {
        let result = CommandResult::from_raw(
            RawOutput {
                rc: 42,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
            "nix build".into(),
        );
        let repr = result.__repr__();
        assert!(repr.contains("rc=42"));
        assert!(repr.contains("nix build"));
    }
}
