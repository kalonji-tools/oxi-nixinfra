# Test services

oxi-nixinfra inspects systemd services. All service checks use `systemctl`
under the hood.

## Check if a service is running

```python
def test_nix_daemon_running(host: Fixture[Host]):
    svc = host.service("nix-daemon")
    assert svc.is_running()
```

## Check if a service is enabled

```python
def test_sshd_enabled(host: Fixture[Host]):
    svc = host.service("sshd")
    assert svc.is_enabled()
```

## Check if a service exists

```python
def test_custom_service_exists(host: Fixture[Host]):
    assert host.service("my-app").exists()
```

## Check if a service is masked

```python
def test_service_not_masked(host: Fixture[Host]):
    assert not host.service("sshd").is_masked()
```

## Validate a service unit file

```python
def test_service_valid(host: Fixture[Host]):
    assert host.service("nix-daemon").is_valid()
```

## Inspect service properties

`properties()` returns a `dict` of all key-value pairs from `systemctl show`:

```python
def test_service_properties(host: Fixture[Host]):
    props = host.service("nix-daemon").properties()
    assert props["Type"] == "notify"
    assert props["Restart"] == "always"
```

## Available methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_running()` | `bool` | `systemctl is-active` |
| `is_enabled()` | `bool` | `systemctl is-enabled` |
| `exists()` | `bool` | `systemctl list-unit-files` |
| `is_masked()` | `bool` | Masked unit check |
| `is_valid()` | `bool` | `systemd-analyze verify` |
| `properties()` | `dict` | `systemctl show` key-value pairs |
