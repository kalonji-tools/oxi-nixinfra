"""oxi-nixinfra: NixOS infrastructure testing library."""

from oxi_nixinfra._oxi_nixinfra import (
    AsyncFile,
    AsyncHost,
    AsyncNixOption,
    AsyncNixPackage,
    AsyncService,
    AsyncSystemInfo,
    AsyncUser,
    CommandResult,
    File,
    Host,
    NixOption,
    NixPackage,
    Service,
    SystemInfo,
    User,
)
from oxi_nixinfra._plugin import is_nixos

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
    "NixOption",
    "AsyncNixOption",
    "SystemInfo",
    "AsyncSystemInfo",
]
