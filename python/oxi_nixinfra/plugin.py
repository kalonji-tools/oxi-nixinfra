"""oxitest plugin entry point for oxi-nixinfra."""

from __future__ import annotations

from oxi_nixinfra._config import oxitest_cli_extension  # noqa: F401


def oxitest_plugin(*, config):
    """Lazy entry point — defers heavy plugin import until activation."""
    from collections.abc import Callable
    from typing import Any

    from oxitest._bridge.result import SkippedResult
    from oxitest.plugin import Plugin

    from oxi_nixinfra import Host, is_nixos

    class NixosWrapper:
        """ExecutionWrapper that skips tests on non-NixOS systems."""

        @property
        def marker(self) -> str:
            return "nixos"

        def wrap(self, test_fn: Callable[[], Any], marker_args: dict[str, Any]) -> Any:
            if not is_nixos():
                return SkippedResult(message="requires NixOS")
            return test_fn()

    class HostProvider:
        """FixtureProvider that injects a Host fixture."""

        def __init__(self, host: str, ssh_config: str | None):
            self._host = host
            self._ssh_config = ssh_config

        @property
        def name(self) -> str:
            return "oxi-nixinfra:host"

        @property
        def fixture_type(self) -> type:
            return Host

        def create(self, ctx: object) -> object:
            return Host._from_config(self._host, ssh_config=self._ssh_config)

        def teardown(self, value: object) -> None:
            pass

    return Plugin(
        fixture_providers=[HostProvider(config.host, config.ssh_config)],
        execution_wrappers=[NixosWrapper()],
    )
