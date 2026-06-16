# NixOS-Native Module Redesign

Milestone: [#5 — NixOS-Native Module Redesign](https://github.com/kalonji-tools/oxi-nixinfra/milestone/5)

## Problem

oxi-nixinfra's modules use generic Linux/systemd abstractions that don't reflect how NixOS manages the system. The most visible symptom: `service.is_enabled()` returns `false` for every declaratively managed NixOS service because `systemctl is-enabled` returns `linked` (exit code 1) instead of `enabled` (exit code 0).

This isn't a one-method bug — it's a design gap. NixOS is a declarative system where packages, services, users, and config files are all declared in Nix and materialized as symlinks into `/nix/store/`. The testing API should reflect this model.

## Design Principle

Interrogate the live system (local, remote, or virtual) via observable filesystem artifacts and commands. No `nixos-option` evaluation for runtime checks (it doesn't work remotely). The key detection pattern across all modules:

```
readlink -f <path> → starts with /nix/store/ → NixOS manages it
```

See `docs/nixos-internals.md` for the full reference on NixOS observable artifacts.

## Module Specifications

### 1. Service

#### Removed methods

| Method | Reason |
|--------|--------|
| `is_enabled()` | Systemd vocabulary, broken on NixOS (`linked` ≠ `enabled`). Replaced by `is_managed()`. |
| `is_masked()` | Subsumed by `enablement_status()`. Users check `enablement_status() == "masked"`. |

#### New methods

| Method | Signature | Implementation | Purpose |
|--------|-----------|----------------|---------|
| `is_managed` | `() -> bool` | `readlink -f /etc/systemd/system/<name>.service`, check if target starts with `/nix/store/` and is not `/dev/null` | Is this service declared in the NixOS configuration? |
| `enablement_status` | `() -> str` | `systemctl show <name> -p UnitFileState --value` | Raw systemd unit file state: `linked`, `enabled`, `masked`, `disabled`, etc. |
| `store_path` | `() -> Option<str>` | `readlink -f /etc/systemd/system/<name>.service`, return if starts with `/nix/store/` | The Nix store path of the unit file, if NixOS-managed. |

#### Unchanged methods

| Method | Implementation | Notes |
|--------|----------------|-------|
| `is_running` | `systemctl is-active <name>` | Runtime state, works correctly. |
| `exists` | `systemctl list-unit-files` scan | "Does systemd know about this unit?" — still valid. |
| `is_valid` | `systemd-analyze verify <name>` | Unit file validation, works correctly. |
| `properties` | `systemctl show <name>` | Full property map, works correctly. |

#### Semantics

- `is_managed()` returns `true` for both `linked` and `enabled` services — both mean NixOS put the unit there.
- `is_managed()` returns `false` for `masked` services (symlink to `/dev/null`), services not in `/etc/systemd/system/`, and services whose unit file doesn't resolve to `/nix/store/`.
- `store_path()` returns `None` for masked services (they resolve to `/dev/null`).

#### Test cases

| Scenario | `is_managed()` | `enablement_status()` | `store_path()` |
|----------|---------------|-----------------------|----------------|
| nix-daemon (linked, socket-activated) | `true` | `"linked"` | `Some("/nix/store/...-nix-2.31.4/lib/systemd/system/nix-daemon.service")` |
| avahi-daemon (enabled, in multi-user.target.wants) | `true` | `"enabled"` | `Some("/nix/store/...-unit-avahi-daemon.service/avahi-daemon.service")` |
| console-getty (masked, /dev/null) | `false` | `"masked"` | `None` |
| sshd (not declared) | `false` | `"not-found"` or error | `None` |

### 2. File

#### New methods

| Method | Signature | Implementation | Purpose |
|--------|-----------|----------------|---------|
| `is_nix_managed` | `() -> bool` | `readlink -f <path>`, check if target starts with `/nix/store/` | Is this file managed by NixOS (symlinked through `/etc/static` or directly into the store)? |
| `store_path` | `() -> Option<str>` | `readlink -f <path>`, return if starts with `/nix/store/` | The resolved Nix store path. |

#### Unchanged methods

All existing POSIX methods are retained: `exists`, `is_file`, `is_directory`, `is_symlink`, `is_executable`, `is_pipe`, `is_socket`, `linked_to`, `user`, `uid`, `group`, `gid`, `mode`, `size`, `content`, `contains`, `md5sum`, `sha256sum`, `listdir`.

These interrogate runtime file state, which remains valid and useful alongside the NixOS provenance check.

#### Test cases

| Path | `is_nix_managed()` | `store_path()` |
|------|-------------------|----------------|
| `/etc/hosts` | `true` | `Some("/nix/store/<hash>-hosts")` |
| `/etc/nix/nix.conf` | `true` | `Some("/nix/store/<hash>-nix.conf")` |
| `/etc/resolv.conf` | `false` | `None` |
| `/etc/passwd` | `false` | `None` |
| `/home/user/notes.txt` | `false` | `None` |

### 3. User

#### New methods

| Method | Signature | Implementation | Purpose |
|--------|-----------|----------------|---------|
| `is_declared` | `() -> bool` | Extract `users-groups.json` path from `/run/current-system/activate` via `grep -oP '/nix/store/[a-z0-9]+-users-groups\.json'`. Read and parse the JSON. Check if user name appears in the `users` array. | Is this user declared in the NixOS configuration? |

#### Implementation detail

The `users-groups.json` lookup requires two commands:
1. `grep -oP '/nix/store/[a-z0-9]+-users-groups\.json' /run/current-system/activate` — extract the manifest path
2. `cat <manifest-path>` — read the JSON

Parsing the JSON happens in Rust. The manifest is a small file (a few KB) containing all declared users and groups.

#### Unchanged methods

All existing methods retained: `exists`, `name`, `uid`, `gid`, `group`, `groups`, `home`, `shell`.

#### Test cases

| User | `exists()` | `is_declared()` |
|------|-----------|-----------------|
| `snregales` (declared normal user) | `true` | `true` |
| `avahi` (declared system user) | `true` | `true` |
| `root` (always exists, declared) | `true` | `true` |
| Imperatively-created user (if `mutableUsers = true`) | `true` | `false` |
| Non-existent user | `false` | `false` |

### 4. SystemInfo

#### Removed methods

| Method | Reason |
|--------|--------|
| `type()` | Generic Linux (`uname -s`). Always `"linux"` — no value for a NixOS-specific plugin. |
| `distribution()` | Generic Linux (`/etc/os-release` `ID`). Always `"nixos"` — no value. |
| `release()` | Generic Linux (`/etc/os-release` `VERSION_ID`). Replaced by `nixos_version()` with more detail. |
| `codename()` | Generic Linux (`/etc/os-release` `VERSION_CODENAME`). Empty on NixOS. |

#### New methods

| Method | Signature | Implementation | Purpose |
|--------|-----------|----------------|---------|
| `nixos_version` | `() -> str` | `cat /run/current-system/nixos-version` | Full NixOS version string (e.g. `25.11.9840.a4bf06618f0b`). |
| `system_profile` | `() -> str` | `readlink /run/current-system` | Store path of the current system generation. |
| `generation_count` | `() -> i32` | Count entries matching `/nix/var/nix/profiles/system-*-link` | Number of system generations available for rollback. |
| `kernel_version` | `() -> str` | Parse `kernel` basename from `boot.json` (e.g. `linux-6.12.83` from the store path) | Running kernel version. |
| `arch` | `() -> str` | `system` field from `boot.json` (e.g. `x86_64-linux`) | System architecture. NixOS-native format instead of raw `uname -m`. |
| `label` | `() -> str` | `label` field from `boot.json` | Human-readable system label (e.g. `NixOS Xantusia 25.11.9840.a4bf06618f0b (Linux 6.12.83)`). |
| `specialisations` | `() -> Vec<str>` | `ls /run/current-system/specialisation/` | List of available system specialisations. |

#### Implementation detail

`boot.json` is read once and cached (same pattern as current `SystemInfo` which caches `/etc/os-release`). The JSON is small and stable within a generation. All methods except `generation_count` read from cached `boot.json` or single-command outputs.

#### Test cases

| Method | Expected output (example) |
|--------|--------------------------|
| `nixos_version()` | `"25.11.9840.a4bf06618f0b"` |
| `system_profile()` | `"/nix/store/l87x...-nixos-system-nixos-25.11.9840.a4bf06618f0b"` |
| `generation_count()` | `16` |
| `kernel_version()` | `"6.12.83"` |
| `arch()` | `"x86_64-linux"` |
| `label()` | `"NixOS Xantusia 25.11.9840.a4bf06618f0b (Linux 6.12.83)"` |
| `specialisations()` | `[]` (empty if none declared) |

### 5. NixPackage — no changes

Already NixOS-native. Queries `nix-store -q --references /run/current-system/sw`. Methods: `is_installed()`, `version()`, `store_path()`. No changes needed.

### 6. NixOption — document limitation

Keep existing API unchanged: `exists()`, `value()`.

Add documentation noting that `nixos-option` is **local-only** — it evaluates the full Nix expression tree and requires access to the system's Nix configuration source. It does not work on remote hosts via SSH.

## Python API Changes

The Python re-exports in `python/oxi_nixinfra/__init__.py` must be updated to reflect:
- Removed methods on Service (`is_enabled`, `is_masked`)
- New methods on all modified modules
- Updated `__all__` if applicable

The plugin (`_plugin.py`) is unaffected — it provides `Host`/`HostProvider`/`NixosWrapper`, not individual module methods.

## Migration Impact

No backward compatibility concerns — the project has no public users and no 1.0.0 release. This is a clean break.

### SNROS downstream

The SNROS migration (PR #56 on kalonji-tools/SNROS) has `is_enabled()` commented out with a TODO referencing this fix. After this redesign ships:
- Replace `service.is_enabled()` with `service.is_managed()`
- Uncomment the assertion

## Scope Boundaries

- **In scope**: All 6 existing modules as specified above.
- **Out of scope**: Home Manager awareness (tracked in #66), new modules (Process, Socket, MountPoint, etc.), NixOS VM CI testing infrastructure (#24).
