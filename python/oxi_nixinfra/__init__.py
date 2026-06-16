"""oxi-nixinfra: NixOS infrastructure testing library."""
from oxi_nixinfra._plugin import is_nixos
from oxi_nixinfra._oxi_nixinfra import (
    Host,
    AsyncHost,
    CommandResult,
    Service,
    AsyncService,
    File,
    AsyncFile,
    User,
    AsyncUser,
    NixPackage,
    AsyncNixPackage,
    NixOption,
    AsyncNixOption,
    SystemInfo,
    AsyncSystemInfo,
)

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
