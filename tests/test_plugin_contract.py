"""Plugin protocol contract tests.

Verify the plugin factory returns a `Plugin` with an `ExecutionWrapper` that
oxitest can call at runtime.  Signature-level conformance (keyword-only
params, keyword names) is validated by ty at check time.

`FixtureProvider` is intentionally not checked via `isinstance` here:
`@runtime_checkable` would over-constrain it — oxitest's loader accesses
optional properties like `scope` and `autouse` via `getattr(..., default)`
(`_fixture_session.py`), so implementations that omit them are valid at
runtime but fail the structural check.  ty catches missing required
members at check time.
"""

from __future__ import annotations


def test_oxitest_plugin_returns_valid_plugin():
    from oxi_nixinfra._config import NixConfig
    from oxi_nixinfra.plugin import oxitest_plugin
    from oxitest.plugin import ExecutionWrapper, Plugin

    result = oxitest_plugin(config=NixConfig())

    assert isinstance(result, Plugin), (
        "oxitest_plugin() must return Plugin — oxitest's loader will reject it"
    )
    assert len(result.fixture_providers) > 0, (
        "no FixtureProvider registered — Host fixture won't be injectable"
    )
    assert len(result.execution_wrappers) > 0, (
        "no ExecutionWrapper registered — @oxitest.mark.nixos won't skip on non-NixOS"
    )

    for wrapper in result.execution_wrappers:
        assert isinstance(wrapper, ExecutionWrapper), (
            f"{type(wrapper).__name__} is missing ExecutionWrapper methods"
            " — oxitest will fail to call it at runtime"
        )


def test_cli_extension_is_discoverable():
    """Phase 1: oxitest can discover CLI extension without loading Rust."""
    from oxi_nixinfra.plugin import oxitest_cli_extension

    assert oxitest_cli_extension.prefix == "nix", (
        "CLI prefix must be 'nix' — flags will be --nix-host, --nix-ssh-config"
    )


def test_config_defaults():
    """NixConfig defaults match expected values."""
    from oxi_nixinfra._config import NixConfig

    config = NixConfig()
    assert config.host == "local://", (
        "default host must be 'local://' for local-only testing"
    )
    assert config.ssh_config is None, (
        "default ssh_config must be None — only set when targeting remote hosts"
    )
