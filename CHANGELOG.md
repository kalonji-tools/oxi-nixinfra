# Changelog

All notable changes to this project will be documented in this file.
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
