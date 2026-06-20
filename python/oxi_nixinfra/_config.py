"""Typed plugin configuration for oxi-nixinfra."""

from dataclasses import dataclass
from typing import Annotated

from oxitest import Both, CliExtension, Conf


@dataclass(frozen=True)
class NixConfig:
    """Configuration for the oxi-nixinfra plugin."""

    host: Annotated[str, Both(short="H", help="Target host", env="OXITEST_HOST")] = (
        "local://"
    )
    ssh_config: Annotated[str | None, Conf(help="SSH config path")] = None


oxitest_cli_extension = CliExtension(prefix="nix", config_type=NixConfig)
