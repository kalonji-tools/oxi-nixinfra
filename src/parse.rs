use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

use crate::backend::BackendError;
use crate::command::RawOutput;

/// Parsed command output with lazy UTF-8 conversion and typed parse helpers.
pub struct CommandOutput {
    raw: RawOutput,
    stdout_cache: OnceLock<String>,
}

impl CommandOutput {
    pub fn from_raw(raw: RawOutput) -> Self {
        Self {
            raw,
            stdout_cache: OnceLock::new(),
        }
    }

    /// Return code of the command.
    pub fn rc(&self) -> i32 {
        self.raw.rc
    }

    /// Trimmed stdout as `&str`. Lazy — computed once on first call.
    pub fn stdout(&self) -> &str {
        self.stdout_cache
            .get_or_init(|| String::from_utf8_lossy(&self.raw.stdout).into_owned())
            .trim()
    }

    /// Untrimmed stdout as owned `String`. For content that must preserve whitespace.
    pub fn stdout_raw(&self) -> String {
        String::from_utf8_lossy(&self.raw.stdout).into_owned()
    }

    /// Raw stdout bytes. For emptiness checks without UTF-8 conversion.
    pub fn stdout_bytes(&self) -> &[u8] {
        &self.raw.stdout
    }

    /// Raw stderr bytes.
    pub fn stderr_bytes(&self) -> &[u8] {
        &self.raw.stderr
    }

    /// Lossy stderr as owned `String`.
    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.raw.stderr).into_owned()
    }

    /// Parse trimmed stdout as an integer type.
    pub fn parse_int<T: FromStr>(&self, field: &str) -> Result<T, BackendError>
    where
        T::Err: std::fmt::Display,
    {
        self.stdout()
            .parse::<T>()
            .map_err(|e| BackendError::Execution(format!("failed to parse {field}: {e}")))
    }

    /// Parse trimmed stdout as `i32` with a given radix (e.g. 8 for octal).
    pub fn parse_int_radix(&self, field: &str, radix: u32) -> Result<i32, BackendError> {
        i32::from_str_radix(self.stdout(), radix)
            .map_err(|e| BackendError::Execution(format!("failed to parse {field}: {e}")))
    }

    /// Parse stdout as JSON.
    fn parse_json(&self) -> Result<serde_json::Value, BackendError> {
        serde_json::from_str(self.stdout())
            .map_err(|e| BackendError::Execution(format!("failed to parse JSON: {e}")))
    }

    /// Extract a top-level JSON string field from stdout.
    #[cfg(test)]
    pub fn json_field(&self, key: &str) -> Result<String, BackendError> {
        let parsed = self.parse_json()?;
        parsed
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| BackendError::Execution(format!("failed to parse {key}: key not found")))
    }

    /// Extract a string field from nested JSON by traversing a path of keys.
    /// For arrays, use `json_array_field` instead.
    pub fn json_nested_field(&self, path: &[&str]) -> Result<String, BackendError> {
        let mut value = self.parse_json()?;
        for &key in path {
            value = value.get(key).cloned().ok_or_else(|| {
                BackendError::Execution(format!(
                    "failed to parse {key}: key not found in path {}",
                    path.join(".")
                ))
            })?;
        }
        value.as_str().map(str::to_owned).ok_or_else(|| {
            BackendError::Execution(format!(
                "failed to parse {}: value is not a string",
                path.join(".")
            ))
        })
    }

    /// Extract a string field from the first element of a JSON array at `array_key`.
    pub fn json_array_field(&self, array_key: &str, field: &str) -> Result<String, BackendError> {
        let parsed = self.parse_json()?;
        parsed
            .get(array_key)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get(field))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| {
                BackendError::Execution(format!(
                    "failed to parse {field}: not found in {array_key}[0]"
                ))
            })
    }

    /// First whitespace-delimited field from trimmed stdout.
    pub fn first_field(&self, context: &str) -> Result<&str, BackendError> {
        self.stdout().split_whitespace().next().ok_or_else(|| {
            BackendError::Execution(format!("failed to parse {context}: empty output"))
        })
    }

    /// Parse stdout as key=value pairs separated by `sep`.
    /// Filters out entries with empty values.
    pub fn kv_pairs(&self, sep: char) -> HashMap<String, String> {
        self.lines()
            .filter_map(|line| line.split_once(sep))
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    /// Lines iterator over trimmed stdout.
    pub fn lines(&self) -> std::str::Lines<'_> {
        self.stdout().lines()
    }

    /// Extract the `index`-th field from a `sep`-delimited line.
    pub fn delimited_field(
        &self,
        sep: char,
        index: usize,
        context: &str,
    ) -> Result<&str, BackendError> {
        self.stdout().split(sep).nth(index).ok_or_else(|| {
            BackendError::Execution(format!(
                "failed to parse {context}: field {index} not found"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_output(stdout: &[u8]) -> CommandOutput {
        CommandOutput::from_raw(RawOutput {
            rc: 0,
            stdout: stdout.to_vec(),
            stderr: vec![],
        })
    }

    fn make_output_with_stderr(stdout: &[u8], stderr: &[u8]) -> CommandOutput {
        CommandOutput::from_raw(RawOutput {
            rc: 1,
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        })
    }

    // --- stdout / stdout_raw ---

    #[test]
    fn test_stdout_trims_whitespace() {
        let out = make_output(b"  hello world  \n");
        assert_eq!(out.stdout(), "hello world");
    }

    #[test]
    fn test_stdout_raw_preserves_content() {
        let out = make_output(b"line1\nline2\n");
        assert_eq!(out.stdout_raw(), "line1\nline2\n");
    }

    #[test]
    fn test_stderr_lossy() {
        let out = make_output_with_stderr(b"", b"error msg\n");
        assert_eq!(out.stderr_lossy(), "error msg\n");
    }

    // --- parse_int ---

    #[test]
    fn test_parse_int_success() {
        let out = make_output(b"1000\n");
        assert_eq!(out.parse_int::<i32>("uid").unwrap(), 1000);
    }

    #[test]
    fn test_parse_int_u64() {
        let out = make_output(b"4096\n");
        assert_eq!(out.parse_int::<u64>("size").unwrap(), 4096);
    }

    #[test]
    fn test_parse_int_error_includes_field() {
        let out = make_output(b"notanumber\n");
        let err = out.parse_int::<i32>("uid").unwrap_err();
        assert!(
            err.to_string().contains("uid"),
            "error should mention the field name"
        );
    }

    // --- parse_int_radix ---

    #[test]
    fn test_parse_int_radix_octal() {
        let out = make_output(b"644\n");
        assert_eq!(out.parse_int_radix("mode", 8).unwrap(), 0o644);
    }

    #[test]
    fn test_parse_int_radix_error_includes_field() {
        let out = make_output(b"zzz\n");
        let err = out.parse_int_radix("mode", 8).unwrap_err();
        assert!(
            err.to_string().contains("mode"),
            "error should mention the field name"
        );
    }

    // --- json_field ---

    #[test]
    fn test_json_field_compact() {
        let out = make_output(br#"{"fstype":"ext4","source":"/dev/sda1"}"#);
        assert_eq!(out.json_field("fstype").unwrap(), "ext4");
        assert_eq!(out.json_field("source").unwrap(), "/dev/sda1");
    }

    #[test]
    fn test_json_field_spaced() {
        let out = make_output(br#"{"fstype": "ext4", "source": "/dev/sda1"}"#);
        assert_eq!(out.json_field("fstype").unwrap(), "ext4");
    }

    #[test]
    fn test_json_field_missing_key() {
        let out = make_output(br#"{"fstype":"ext4"}"#);
        let err = out.json_field("missing").unwrap_err();
        assert!(
            err.to_string().contains("missing"),
            "error should mention the key name"
        );
    }

    #[test]
    fn test_json_field_nested_object() {
        let out = make_output(br#"{"outer":{"inner":"value"},"target":"found"}"#);
        assert_eq!(out.json_field("target").unwrap(), "found");
    }

    #[test]
    fn test_json_field_key_as_substring() {
        let out = make_output(br#"{"fstype_extra":"btrfs","fstype":"ext4"}"#);
        assert_eq!(out.json_field("fstype").unwrap(), "ext4");
    }

    #[test]
    fn test_json_field_invalid_json() {
        let out = make_output(b"not json at all");
        let err = out.json_field("key").unwrap_err();
        assert!(
            err.to_string().contains("failed to parse JSON"),
            "should report JSON parse error"
        );
    }

    // --- first_field ---

    #[test]
    fn test_first_field_success() {
        let out = make_output(b"d41d8cd98f00b204e9800998ecf8427e  /dev/null\n");
        assert_eq!(
            out.first_field("md5sum").unwrap(),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
    }

    #[test]
    fn test_first_field_empty_output() {
        let out = make_output(b"\n");
        let err = out.first_field("md5sum").unwrap_err();
        assert!(
            err.to_string().contains("md5sum"),
            "error should mention the context"
        );
    }

    // --- kv_pairs ---

    #[test]
    fn test_kv_pairs_equals_separator() {
        let out =
            make_output(b"Type=simple\nExecStart=/bin/foo\nDescription=\nActiveState=active\n");
        let pairs = out.kv_pairs('=');
        assert_eq!(pairs.get("Type").unwrap(), "simple");
        assert_eq!(pairs.get("ExecStart").unwrap(), "/bin/foo");
        assert_eq!(pairs.get("ActiveState").unwrap(), "active");
        assert!(
            !pairs.contains_key("Description"),
            "empty values should be filtered"
        );
    }

    // --- lines ---

    #[test]
    fn test_lines_iteration() {
        let out = make_output(b"line1\nline2\nline3\n");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    // --- delimited_field ---

    #[test]
    fn test_delimited_field_colon_separated() {
        let out = make_output(b"testuser:x:1000:1000:Test User:/home/testuser:/bin/bash\n");
        assert_eq!(
            out.delimited_field(':', 5, "home").unwrap(),
            "/home/testuser"
        );
        assert_eq!(out.delimited_field(':', 6, "shell").unwrap(), "/bin/bash");
    }

    #[test]
    fn test_delimited_field_out_of_bounds() {
        let out = make_output(b"a:b:c\n");
        let err = out.delimited_field(':', 10, "field10").unwrap_err();
        assert!(
            err.to_string().contains("field10"),
            "error should mention the context"
        );
    }

    // --- rc / stdout_bytes ---

    #[test]
    fn test_rc() {
        let out = CommandOutput::from_raw(RawOutput {
            rc: 42,
            stdout: vec![],
            stderr: vec![],
        });
        assert_eq!(out.rc(), 42);
    }

    #[test]
    fn test_stdout_bytes() {
        let out = make_output(b"raw bytes");
        assert_eq!(out.stdout_bytes(), b"raw bytes");
    }
}
