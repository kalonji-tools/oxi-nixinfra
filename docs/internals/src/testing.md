# Testing

## Rust unit tests

Mock the `Backend` trait with `MockBackend` to return canned `RawOutput`.
Test output parsing, not command execution.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;

    #[tokio::test]
    async fn test_service_is_running() {
        let mock = MockBackend::new(vec![
            RawOutput { rc: 0, stdout: b"active\n".to_vec(), stderr: vec![] },
        ]);
        let inner = HostInner::with_backend(Box::new(mock));
        assert!(is_running_impl(&inner, "nix-daemon").await.unwrap());
    }
}
```

Run with:
```bash
just test-rust
```

### What to test in Rust

- Connection string parsing (`host.rs`)
- Module output parsing (e.g., `systemctl show` → dict)
- `CommandResult` construction and methods
- Store path version parsing (`nix_package.rs`)
- `/etc/os-release` parsing (`system_info.rs`)

## Python integration tests

Run against the local NixOS machine via `LocalBackend`. These are real
commands — not mocked.

```python
from oxitest import Fixture
from oxi_nixinfra import Host

def test_service_running(host: Fixture[Host]):
    assert host.service("nix-daemon").is_running()
```

Run with:
```bash
just test
```

### NixOS marker

Tests that require NixOS should use the `@oxitest.mark.nixos` marker.
These auto-skip on non-NixOS systems (e.g., CI runners):

```python
import oxitest

@oxitest.mark.nixos
def test_nix_specific(host: Fixture[Host]):
    assert host.nix_package("nix").is_installed()
```

## Quality checks

```bash
# All static checks
just check

# Full pre-push gate (clean + check + test-rust + build + test)
just preflight
```
