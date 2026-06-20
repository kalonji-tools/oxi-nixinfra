"""oxi-nixinfra: NixOS infrastructure testing library."""

import functools
from pathlib import Path

from oxi_nixinfra._oxi_nixinfra import (
    AsyncEnvironment,
    AsyncFile,
    AsyncHost,
    AsyncMountPoint,
    AsyncNixPackage,
    AsyncProcess,
    AsyncService,
    AsyncSocket,
    AsyncSysctl,
    AsyncSystemInfo,
    AsyncUser,
    CommandResult,
    Environment,
    File,
    Host,
    MountPoint,
    NixPackage,
    Process,
    Service,
    Socket,
    Sysctl,
    SystemInfo,
    User,
)


@functools.cache
def is_nixos() -> bool:
    """Detect NixOS by reading /etc/os-release."""
    try:
        with Path("/etc/os-release").open() as f:
            return any(line.strip() == "ID=nixos" for line in f)
    except FileNotFoundError:
        return False


__all__ = [
    "is_nixos",
    "Host",
    "AsyncHost",
    "CommandResult",
    "Service",
    "AsyncService",
    "File",
    "AsyncFile",
    "User",
    "AsyncUser",
    "NixPackage",
    "AsyncNixPackage",
    "SystemInfo",
    "AsyncSystemInfo",
    "Process",
    "AsyncProcess",
    "Socket",
    "AsyncSocket",
    "MountPoint",
    "AsyncMountPoint",
    "Sysctl",
    "AsyncSysctl",
    "Environment",
    "AsyncEnvironment",
]
