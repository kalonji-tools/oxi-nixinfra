# Why NixOS only

oxi-nixinfra is NixOS-specific by design. It does not support other Linux
distributions, BSDs, macOS, or Windows.

## The rationale

Traditional infrastructure testing tools like testinfra support multiple
platforms through runtime detection and abstraction layers. This comes with
costs:

- **Platform polymorphism** — every module needs conditional logic for
  different package managers, init systems, and file layouts.
- **Lowest common denominator** — the API can only expose features that
  exist across all platforms.
- **Testing burden** — every module must be tested on every supported
  platform.

NixOS eliminates these problems. The system is fully declarative, uses
systemd exclusively, has a single package manager, and follows predictable
file system conventions. By targeting NixOS alone, every module can be
focused and precise.

## What this means in practice

- **Systemd only** — the Service module uses `systemctl` directly with no
  init system detection.
- **GNU coreutils only** — the File module uses `stat -Lc` flags that don't
  exist on BSD.
- **Nix store queries** — the NixPackage module queries
  `/run/current-system/sw` which only exists on NixOS.
- **No platform detection** — no runtime checks, no fallback paths, no
  `if platform == ...` branches.

## Non-goals

- Multi-distro or cross-platform support
- Config management tool integration (Ansible, Salt, Puppet)
- Container backends (Docker, Podman, Kubernetes)
