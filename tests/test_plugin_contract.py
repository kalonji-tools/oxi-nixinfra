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


def test_helper_extracts_fixture_provider_methods():
    from oxitest.plugin import FixtureProvider

    methods = _protocol_methods(FixtureProvider)
    # FixtureProvider: name (prop), fixture_type (prop),
    # create(ctx), teardown(value)
    assert "name" in methods, "FixtureProvider must have 'name'"
    assert "fixture_type" in methods, "FixtureProvider must have 'fixture_type'"
    assert "create" in methods, "FixtureProvider must have 'create'"
    assert "teardown" in methods, "FixtureProvider must have 'teardown'"
    assert len(methods) == 4, (
        f"FixtureProvider should have 4 methods, got {len(methods)}: {list(methods)}"
    )


def test_host_provider_conforms_to_fixture_provider():
    from oxi_nixinfra._plugin import HostProvider
    from oxitest.plugin import FixtureProvider

    expected = _protocol_methods(FixtureProvider)
    for method_name, expected_arity in expected.items():
        attr = getattr(HostProvider, method_name, None)
        assert attr is not None, (
            f"HostProvider is missing '{method_name}' required by FixtureProvider"
        )
        if isinstance(
            inspect.getattr_static(HostProvider, method_name, None), property
        ):
            continue  # properties have no callable signature to check
        sig = inspect.signature(attr)
        params = [p for p in sig.parameters.values() if p.name != "self"]
        assert len(params) == expected_arity, (
            f"HostProvider.{method_name} has {len(params)} params, "
            f"FixtureProvider expects {expected_arity}"
        )


def test_nixos_wrapper_conforms_to_execution_wrapper():
    from oxi_nixinfra._plugin import NixosWrapper
    from oxitest.plugin import ExecutionWrapper

    expected = _protocol_methods(ExecutionWrapper)
    for method_name, expected_arity in expected.items():
        attr = getattr(NixosWrapper, method_name, None)
        assert attr is not None, (
            f"NixosWrapper is missing '{method_name}' required by ExecutionWrapper"
        )
        if isinstance(
            inspect.getattr_static(NixosWrapper, method_name, None), property
        ):
            continue
        sig = inspect.signature(attr)
        params = [p for p in sig.parameters.values() if p.name != "self"]
        assert len(params) == expected_arity, (
            f"NixosWrapper.{method_name} has {len(params)} params, "
            f"ExecutionWrapper expects {expected_arity}"
        )


def test_oxitest_plugin_returns_valid_plugin():
    from oxi_nixinfra._plugin import oxitest_plugin
    from oxitest.plugin import ExecutionWrapper, FixtureProvider, Plugin

    result = oxitest_plugin()

    assert isinstance(result, Plugin), (
        f"oxitest_plugin() returned {type(result).__name__}, expected Plugin"
    )
    assert len(result.fixture_providers) > 0, (
        "oxitest_plugin().fixture_providers is empty"
    )
    assert len(result.execution_wrappers) > 0, (
        "oxitest_plugin().execution_wrappers is empty"
    )

    # Every provider must satisfy its protocol (structural check)
    for provider in result.fixture_providers:
        assert isinstance(provider, FixtureProvider), (
            f"{type(provider).__name__} does not satisfy FixtureProvider protocol"
        )
    for wrapper in result.execution_wrappers:
        assert isinstance(wrapper, ExecutionWrapper), (
            f"{type(wrapper).__name__} does not satisfy ExecutionWrapper protocol"
        )
