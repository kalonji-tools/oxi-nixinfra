# Test NixOS options

The `NixOption` module queries the running system's evaluated NixOS
configuration.

## Check if an option exists

```python
def test_ssh_option_exists(host: Fixture[Host]):
    opt = host.nix_option("services.openssh.enable")
    assert opt.exists()
```

## Get an option's value

```python
def test_ssh_enabled(host: Fixture[Host]):
    opt = host.nix_option("services.openssh.enable")
    assert opt.value() is True
```

```python
def test_timezone(host: Fixture[Host]):
    tz = host.nix_option("time.timeZone")
    assert tz.value() == "America/New_York"
```

## Available methods

| Method | Returns | Description |
|--------|---------|-------------|
| `value()` | `object` | The option's evaluated value |
| `exists()` | `bool` | Whether the option path is valid |
