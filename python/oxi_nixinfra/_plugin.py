"""oxitest plugin entry point for oxi-nixinfra."""

from __future__ import annotations

import functools
import os
import warnings
from collections.abc import Callable
from pathlib import Path
from typing import Any

from oxitest.plugin import Plugin


@functools.cache
def is_nixos() -> bool:
    """Detect NixOS by reading /etc/os-release."""
    try:
        with Path("/etc/os-release").open() as f:
            return any(line.strip() == "ID=nixos" for line in f)
    except FileNotFoundError:
        return False


class NixosWrapper:
    """ExecutionWrapper that skips tests on non-NixOS systems."""

    @property
    def marker(self) -> str:
        return "nixos"

    def wrap(self, test_fn: Callable[[], Any], marker_args: dict[str, Any]) -> Any:
        if not is_nixos():
            from oxitest._bridge.result import SkippedResult

            return SkippedResult(message="requires NixOS")
        return test_fn()


class HostProvider:
    """FixtureProvider that injects a Host fixture."""

    _VALID_CONFIG_KEYS = frozenset({"host", "ssh_config"})

    def __init__(self, config: dict[str, Any] | None):
        self._config = config or {}
        unknown = set(self._config) - self._VALID_CONFIG_KEYS
        if unknown:
            warnings.warn(
                f"oxi-nixinfra: unrecognized config keys: {', '.join(sorted(unknown))}."
                f" Valid keys: {', '.join(sorted(self._VALID_CONFIG_KEYS))}",
                UserWarning,
                stacklevel=2,
            )

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
            os.environ.get("OXITEST_HOST") or self._config.get("host") or "local://"
        )
        ssh_config = self._config.get("ssh_config")
        return Host._from_config(host_str, ssh_config=ssh_config)

    def teardown(self, value: object) -> None:
        pass


def oxitest_plugin(*, config: dict[str, Any] | None = None) -> Plugin:
    """Entry point called by oxitest's plugin loader."""
    return Plugin(
        fixture_providers=[HostProvider(config)],
        execution_wrappers=[NixosWrapper()],
    )
