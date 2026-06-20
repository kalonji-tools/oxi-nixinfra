# Changelog

All notable changes to this project will be documented in this file.
## [0.4.0] - 2026-06-20

### Bug Fixes


- Sync flake.nix version with Cargo.toml (#110)
- Renumber duplicate ADR 0001 to 0002 (#112)
- Escape sed quotes in flake.nix version sync (#130)

### Features


- Add CommandOutput type with parse helpers (#96)
- Wire HostInner::execute to return CommandOutput and migrate all modules (#96)
- Cache BootInfo via OnceLock to eliminate triple command execution (#97)
- Add type annotations to plugin methods (#117)
- Warn on unrecognized plugin config keys (#117)
- Implement SSH config file support (#115)

## [0.3.0] - 2026-06-19

### Features


- MkDocs setup and configuration (#76)
- Internals book setup with mdbook (#81)
- Add nix_module! and register_modules! proc macros (#25)
- Migrate all modules and Host to proc macros (#25)
- Add Process, Socket, MountPoint, Sysctl, Environment modules (#17, #18, #19, #20, #21)
- Conftest reads OXITEST_HOST env var (#93)
- Configure VM for full integration tests (#93)
- Run full integration suite in NixOS VM (#93)

## [0.2.0] - 2026-06-16

### Features


- NixOS-native redesign — is_managed, enablement_status, store_path (#67)
- Add is_nix_managed and store_path (#68)
- Replace generic Linux with NixOS-native system info (#70)
- Add is_declared for NixOS user provenance detection (#69)

## [0.1.0] - 2026-06-16

### Bug Fixes


- Use accounts-daemon for is_enabled test (#14)
- Add initial_tag to cliff.toml for first release (#64)

### Features


- Add CommandResult and RawOutput structs (#2)
- Add Backend trait, BackendError, and MockBackend (#3, #13)
- Add LocalBackend using tokio::process::Command (#3)
- Add SshBackend using openssh crate (#4)
- Add helper utilities — wrap_sync, backend_err_to_py, extract_args (#5)
- Add Host, AsyncHost, connection parsing, OnceLock cache (#5)
- Add File and AsyncFile modules (GNU coreutils) (#7)
- Add User and AsyncUser modules (#8)
- Add SystemInfo and AsyncSystemInfo modules (#11)
- Add NixPackage and AsyncNixPackage modules (#9)
- Add NixOption and AsyncNixOption modules (#10)
- Update Python re-exports for all modules (#1)
- Add Service and AsyncService modules (#6)
- Wire up oxitest FixtureProvider plugin (#12)
- Add nixos mark via ExecutionWrapper protocol (#35, #36)
- Strict linting — clippy, ruff, oxitest strict (#37)
- Add prek git hooks for pre-commit and pre-push (#39)
- Add git-cliff for changelog generation (#42)
