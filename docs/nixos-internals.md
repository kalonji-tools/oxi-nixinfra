# NixOS Internals: How the System Works

Reference document for oxi-nixinfra developers. Everything here was verified on a live NixOS 25.11 system and describes observable artifacts that can be interrogated locally, remotely, or in VMs.

## The Declarative Model

NixOS has one source of truth: the Nix configuration (`configuration.nix` or flake modules). Users don't imperatively install packages, enable services, or create users. They declare the desired state in Nix and run `nixos-rebuild switch`, which atomically builds a new system generation and switches to it.

The live system is a graph of symlinks into `/nix/store/` — an immutable, content-addressed store. Every package, service unit, config file, and system tool lives there. The "running system" is just a set of pointers.

## System Profile

The current system generation lives at `/run/current-system/`, which is a symlink to a store path:

```
/run/current-system -> /nix/store/<hash>-nixos-system-<hostname>-<version>
```

Contents of `/run/current-system/`:

| Path | Purpose |
|------|---------|
| `activate` | Shell script that applies the generation (creates users, links /etc, reloads systemd) |
| `boot.json` | Structured JSON: kernel, initrd, kernel params, system arch, label, specialisations |
| `etc/` | The `/etc` overlay — all NixOS-managed config files |
| `sw/` | The system path — all declared packages (`environment.systemPackages`) |
| `systemd/` | Systemd units, binaries |
| `nixos-version` | Plain text, e.g. `25.11.9840.a4bf06618f0b` |
| `kernel` | Symlink to the kernel bzImage in `/nix/store/` |
| `kernel-params` | Kernel command line parameters |
| `specialisation/` | Named specialisations (alternative system configs) |

### boot.json

```json
{
  "org.nixos.bootspec.v1": {
    "init": "/nix/store/...-nixos-system-.../init",
    "initrd": "/nix/store/...-initrd-linux-6.12.83/initrd",
    "kernel": "/nix/store/...-linux-6.12.83/bzImage",
    "kernelParams": ["loglevel=4", "lsm=landlock,yama,bpf"],
    "label": "NixOS Xantusia 25.11.9840.a4bf06618f0b (Linux 6.12.83)",
    "system": "x86_64-linux",
    "toplevel": "/nix/store/...-nixos-system-..."
  },
  "org.nixos.specialisation.v1": {},
  "org.nixos.systemd-boot": { "sortKey": "nixos" }
}
```

### Generations

Every `nixos-rebuild switch` creates a new generation:

```
/nix/var/nix/profiles/system -> system-16-link
/nix/var/nix/profiles/system-16-link -> /nix/store/<hash>-nixos-system-...
/nix/var/nix/profiles/system-15-link -> /nix/store/<hash>-nixos-system-...
...
```

Old generations remain available for rollback until garbage-collected.

## Services

### How NixOS manages systemd services

NixOS does not use `systemctl enable`. Instead:

1. A NixOS module declares a service (e.g. `services.avahi.enable = true`)
2. The module system generates a systemd unit file and places it in the store
3. `nixos-rebuild switch` creates `/etc/systemd/system/` as a directory of symlinks into `/nix/store/`
4. Systemd sees these as "linked" units (symlinked into the unit search path but not installed via `systemctl enable`)

### The enabled vs linked distinction

On a typical NixOS system:

| UnitFileState | Count (example) | Meaning |
|---------------|-----------------|---------|
| `linked` | 80 | Unit file symlinked to `/nix/store/`, not in any `.wants/` dir. Started by socket activation, dependency, or explicit `systemctl start`. |
| `enabled` | 46 | Unit file symlinked to `/nix/store/` AND has a symlink in a `.wants/` dir (e.g. `multi-user.target.wants/`). NixOS wired it to start at boot. |
| `masked` | 6 | Symlinked to `/dev/null`. NixOS explicitly disabled it. |
| `alias` | 14 | Symlink to another unit name. |
| `generated` | (rare) | Created by systemd generators at boot. |

**Critical insight:** Both `linked` and `enabled` mean "NixOS manages this service." The difference is only in *how* systemd activates it:
- `enabled` = starts via target dependency (WantedBy in `.wants/` dir)
- `linked` = starts via socket activation, explicit dependency, or on-demand

`systemctl is-enabled` returns exit code 0 only for `enabled` and `static`, returning exit code 1 for `linked`. This is why the naive `rc == 0` check fails on NixOS — most NixOS services are `linked`.

### How to detect if a service is NixOS-managed

```bash
readlink -f /etc/systemd/system/<name>.service
```

- Resolves to `/nix/store/...` → NixOS manages it
- Resolves to `/dev/null` → NixOS explicitly masked it
- Does not exist → NixOS did not declare it

### How .wants/ directories work on NixOS

The `.wants/` symlinks are also NixOS-managed (part of the read-only `/etc/systemd/system/` directory):

```
/etc/systemd/system/multi-user.target.wants/avahi-daemon.service -> ../avahi-daemon.service
/etc/systemd/system/sockets.target.wants/nix-daemon.socket -> ../nix-daemon.socket
```

NixOS modules that set `wantedBy = ["multi-user.target"]` get a symlink here. Modules that use socket activation (like nix-daemon) get their `.socket` in `sockets.target.wants/` instead — the `.service` itself stays `linked`.

### Override directories

NixOS generates `.d/` override directories for services that need environment variables, paths, or extra dependencies:

```
/etc/systemd/system/accounts-daemon.service.d/overrides.conf
```

These typically set `Environment=`, `PATH=`, `LOCALE_ARCHIVE=`, etc. with Nix store paths.

## Packages

### System packages

Declared packages live in `/run/current-system/sw/`:

```
/run/current-system/sw/bin/git -> /nix/store/<hash>-git-2.44.0/bin/git
```

To check if a package is in the system profile:

```bash
nix-store -q --references /run/current-system/sw
```

This lists all store paths that the system profile references. Package presence and version can be parsed from the store path basename: `/nix/store/<32-char-hash>-<name>-<version>`.

### Per-user packages

Users may also have packages in their per-user profile:

```
/etc/profiles/per-user/<username> -> /etc/static/profiles/per-user/<username>
                                   -> /nix/store/<hash>-user-environment
```

## Files and /etc

### How NixOS manages /etc

NixOS generates `/etc` entries through `environment.etc.*` options. The mechanism:

```
/etc/hosts -> /etc/static/hosts -> /nix/store/<hash>-hosts
/etc/nix/nix.conf -> /etc/static/nix/nix.conf -> /nix/store/<hash>-nix.conf
```

`/etc/static` is itself a symlink into the store:

```
/etc/static -> /nix/store/<hash>-etc/etc
```

### How to detect if a file is NixOS-managed

```bash
readlink -f /path/to/file
```

- Resolves to `/nix/store/...` → NixOS manages it
- Resolves to itself or a non-store path → not NixOS-managed (mutable, runtime-generated, or user-created)

Examples:

| File | Resolves to | Managed? |
|------|------------|----------|
| `/etc/hosts` | `/nix/store/<hash>-hosts` | Yes |
| `/etc/hostname` | `/nix/store/<hash>-etc-hostname` | Yes |
| `/etc/nix/nix.conf` | `/nix/store/<hash>-nix.conf` | Yes |
| `/etc/resolv.conf` | `/etc/resolv.conf` | No (runtime) |
| `/etc/passwd` | `/etc/passwd` | No (mutable) |

### Mutable vs immutable

Files inside `/nix/store/` are immutable (read-only filesystem). Files that NixOS needs to be mutable (like `/etc/passwd` when `users.mutableUsers = true`) are *not* symlinked through `/etc/static` — they live directly in `/etc/` as regular files.

## Users and Groups

### The users-groups.json manifest

NixOS declares all users and groups in a JSON manifest stored in the Nix store. The activation script (`/run/current-system/activate`) references it:

```bash
grep -oP '/nix/store/[a-z0-9]+-users-groups\.json' /run/current-system/activate
```

The manifest contains the full declared state:

```json
{
  "users": [
    {
      "name": "snregales",
      "isSystemUser": false,
      "createHome": true,
      "home": "/home/snregales",
      "group": "users",
      "shell": "/run/current-system/sw/bin/bash",
      "description": "Sharlon N. Regales",
      "uid": null,
      "autoSubUidGidRange": true,
      "subUidRanges": [],
      "subGidRanges": []
    }
  ],
  "groups": [
    { "name": "wheel", "gid": null, "members": ["snregales"] }
  ],
  "mutableUsers": true
}
```

### Declared vs imperative users

- `mutableUsers = true` (default): NixOS creates declared users on activation, but allows imperative changes (`useradd`, password changes). `/etc/passwd` is mutable.
- `mutableUsers = false`: NixOS fully owns `/etc/passwd` and `/etc/group`. Any imperative changes are overwritten on next `nixos-rebuild switch`.

### How to detect if a user is NixOS-declared

1. Extract the `users-groups.json` path from `/run/current-system/activate`
2. Parse the JSON
3. Check if the user's name appears in the `users` array

Additional signals:
- NixOS-declared users typically have shells pointing to `/run/current-system/sw/bin/` (a store path)
- Imperatively-created users typically have shells like `/bin/bash` or `/usr/bin/zsh`

## NixOS Options (nixos-option)

`nixos-option <path>` evaluates the full Nix expression tree to query config values. It returns:

```
Value:
  true

Default:
  false

Type:
  boolean

Description:
  Whether to enable the OpenSSH secure shell daemon...

Declared by:
  /nix/store/...-nixos-25.11/nixos/.../sshd.nix
```

### Limitations

- **Slow**: evaluates the entire Nix configuration on every call
- **Local only**: requires access to the system's Nix configuration. Does not work on remote hosts via SSH — there's no way to evaluate another machine's Nix expressions without having the configuration source
- **Option paths != service names**: `services.avahi.enable` → `avahi-daemon.service`, `networking.networkmanager.enable` → `NetworkManager.service`. There is no programmatic mapping — the NixOS module system wires this internally

## Home Manager (deferred — see #66)

Home Manager manages user-level packages and dotfiles. Two integration modes:

- **NixOS module**: `home-manager.users.<name>` in `configuration.nix`. Activates during `nixos-rebuild`. Referenced in `/run/current-system/activate`.
- **Standalone**: `home-manager switch`. Manages `~/.local/state/nix/profiles/home-manager`. Independent of NixOS.

Both modes create user dotfiles as symlinks into `/nix/store/`. Detection deferred to a future milestone.
