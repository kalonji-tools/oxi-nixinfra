"""Plugin protocol contract tests.

Verify that HostProvider and NixosWrapper satisfy oxitest's FixtureProvider
and ExecutionWrapper protocols.  Catches method name / arity drift that would
otherwise surface as a runtime TypeError deep inside a test run.
"""

from __future__ import annotations

import inspect


def _protocol_methods(protocol_cls: type) -> dict[str, int]:
    """Return {method_name: param_count} for every non-dunder method/property.

    Properties count as 0-param (only ``self``).
    Regular methods count positional-or-keyword params excluding ``self``.
    """
    methods: dict[str, int] = {}
    for name, obj in inspect.getmembers(protocol_cls):
        if name.startswith("_"):
            continue
        # Properties: declared with @property in the protocol body
        if isinstance(inspect.getattr_static(protocol_cls, name, None), property):
            methods[name] = 0
            continue
        if callable(obj):
            sig = inspect.signature(obj)
            # Exclude 'self' — first param of unbound methods
            params = [p for p in sig.parameters.values() if p.name != "self"]
            methods[name] = len(params)
    return methods


def _get_plugin_classes():
    """Import and return (HostProvider, NixosWrapper) by invoking the plugin."""
    from oxi_nixinfra._config import NixConfig
    from oxi_nixinfra.plugin import oxitest_plugin

    plugin = oxitest_plugin(config=NixConfig())
    host_provider = plugin.fixture_providers[0]
    nixos_wrapper = plugin.execution_wrappers[0]
    return type(host_provider), type(nixos_wrapper)


def test_helper_extracts_fixture_provider_methods():
    from oxitest.plugin import FixtureProvider

    methods = _protocol_methods(FixtureProvider)
    # FixtureProvider: name (prop), fixture_type (prop),
    # create(ctx), teardown(value)
    assert "name" in methods, (
        "_protocol_methods helper broke — update it to match FixtureProvider"
    )
    assert "fixture_type" in methods, (
        "_protocol_methods helper broke — update it to match FixtureProvider"
    )
    assert "create" in methods, (
        "_protocol_methods helper broke — update it to match FixtureProvider"
    )
    assert "teardown" in methods, (
        "_protocol_methods helper broke — update it to match FixtureProvider"
    )
    assert len(methods) == 4, (
        "FixtureProvider protocol changed — update _protocol_methods"
        " and conformance tests to match"
    )


def test_host_provider_conforms_to_fixture_provider():
    from oxitest.plugin import FixtureProvider

    HostProvider, _ = _get_plugin_classes()

    expected = _protocol_methods(FixtureProvider)
    for method_name, expected_arity in expected.items():
        attr = getattr(HostProvider, method_name, None)
        assert attr is not None, (
            f"FixtureProvider protocol added '{method_name}'"
            " — implement it on HostProvider or oxitest will TypeError at runtime"
        )
        if isinstance(
            inspect.getattr_static(HostProvider, method_name, None), property
        ):
            continue  # properties have no callable signature to check
        sig = inspect.signature(attr)
        params = [p for p in sig.parameters.values() if p.name != "self"]
        assert len(params) == expected_arity, (
            f"FixtureProvider.{method_name} signature changed"
            " — update HostProvider to match or oxitest will TypeError at runtime"
        )


def test_nixos_wrapper_conforms_to_execution_wrapper():
    from oxitest.plugin import ExecutionWrapper

    _, NixosWrapper = _get_plugin_classes()

    expected = _protocol_methods(ExecutionWrapper)
    for method_name, expected_arity in expected.items():
        attr = getattr(NixosWrapper, method_name, None)
        assert attr is not None, (
            f"ExecutionWrapper protocol added '{method_name}'"
            " — implement it on NixosWrapper or oxitest will TypeError at runtime"
        )
        if isinstance(
            inspect.getattr_static(NixosWrapper, method_name, None), property
        ):
            continue
        sig = inspect.signature(attr)
        params = [p for p in sig.parameters.values() if p.name != "self"]
        assert len(params) == expected_arity, (
            f"ExecutionWrapper.{method_name} signature changed"
            " — update NixosWrapper to match or oxitest will TypeError at runtime"
        )


def test_oxitest_plugin_returns_valid_plugin():
    from oxi_nixinfra._config import NixConfig
    from oxi_nixinfra.plugin import oxitest_plugin
    from oxitest.plugin import ExecutionWrapper, FixtureProvider, Plugin

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

    # Every provider must satisfy its protocol (structural check)
    for provider in result.fixture_providers:
        assert isinstance(provider, FixtureProvider), (
            f"{type(provider).__name__} is missing FixtureProvider methods"
            " — oxitest will fail to call it at runtime"
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
