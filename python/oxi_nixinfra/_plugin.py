"""oxitest plugin entry point for oxi-nixinfra."""
from __future__ import annotations

import functools
import os

from oxitest.plugin import Plugin


@functools.cache
def _is_nixos() -> bool:
    """Detect NixOS by reading /etc/os-release."""
    try:
        with open("/etc/os-release") as f:
            return any(line.strip() == "ID=nixos" for line in f)
    except FileNotFoundError:
        return False


class NixosWrapper:
    """ExecutionWrapper that skips tests on non-NixOS systems."""

    @property
    def marker(self) -> str:
        return "nixos"

    def wrap(self, test_fn, marker_args):
        if not _is_nixos():
            from oxitest._bridge.result import SkippedResult

            return SkippedResult(message="requires NixOS")
        return test_fn()


class HostProvider:
    """FixtureProvider that injects a Host fixture."""

    def __init__(self, config: dict | None):
        self._config = config or {}

    @property
    def name(self) -> str:
        return "oxi-nixinfra:host"

    @property
    def fixture_type(self) -> type:
        from oxi_nixinfra import Host

        return Host

    def create(self, ctx: object) -> object:
        from oxi_nixinfra import Host

        host_str = (
            os.environ.get("OXITEST_HOST")
            or self._config.get("host")
            or "local://"
        )
        ssh_config = self._config.get("ssh_config")
        return Host._from_config(host_str, ssh_config=ssh_config)

    def teardown(self, value: object) -> None:
        pass


def oxitest_plugin(*, config: dict | None = None) -> Plugin:
    """Entry point called by oxitest's plugin loader."""
    return Plugin(
        fixture_providers=[HostProvider(config)],
        execution_wrappers=[NixosWrapper()],
    )
