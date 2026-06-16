# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

oxi-nixinfra is a NixOS infrastructure testing library, built as an oxitest plugin with a Rust (PyO3) core. It executes commands on local or remote NixOS hosts and inspects system state through typed modules.

NixOS-specific by design — no multi-distro support.

## Commands

```bash
# Enter development shell (provides cargo, python, maturin, just)
devenv shell

# Build the Rust extension (required before running Python tests)
just build

# Run Rust unit tests
just test-rust

# Run Python integration tests (requires just build first)
just test

# Run all static checks (ruff, cargo fmt, clippy, codespell)
just check

# Full pre-push gate (clean + check + test-rust + build + test)
just preflight

# Format code
just fmt

# Show all available recipes
just
```

## Workflow

### New ideas → Grill → Issues → Spec → Plan → Implement → Merge

**1. Grill new ideas.** Any new feature, concept, or design direction MUST go through `grill-with-docs` before anything else. This ensures ideas are stress-tested against the existing domain model and documented decisions before committing to them.

**2. Create issues.** Once an idea survives grilling and is deemed worth implementing, create GitHub issues. Organize into milestones if the work spans multiple issues. Every issue MUST be triaged. Apply one **category label** (`bug` or `enhancement`) and one **component label** (`rust`, `python`, or `infra`) to each issue.

**3. Triage issues.** Every issue gets a **state label** reflecting its triage status. Apply exactly one:
- `needs-triage` — maintainer needs to evaluate
- `needs-info` — waiting on reporter for more information
- `ready-for-agent` — fully specified, ready for an AFK agent
- `ready-for-human` — needs human implementation
- `wontfix` — will not be actioned

**4. Spec every issue.** By the time a PR is created, every issue in that PR MUST have a design spec. Specs can be written when the issue is picked up or ahead of time — but never skipped. Use the `superpowers:brainstorming` skill for spec design. Post each issue's spec section as a comment on that issue. When issues share a grouped spec, post only the section relevant to each issue — not the entire spec on every issue.

**5. Create a draft PR.** Push the branch and open a draft PR before any implementation begins. This gives reviewers a chance to evaluate the approach early.

**6. Plan before implementing.** Use the `superpowers:writing-plans` skill. Multiple issues can be grouped into one plan if they are tightly coupled or logically sequential. The plan MUST be posted as a comment on the PR — never on individual issues.

**7. Implement via subagents or inline.** Use `superpowers:subagent-driven-development` or `superpowers:executing-plans`.

**8. Merge rules.**
- **Never push directly to main.** All changes go through pull requests.
- **Never merge without approval.** Wait for either a GitHub review approval or an explicit user command (e.g., "merge", "merge rebase delete branch"). Do not auto-merge after CI passes.
- Only `--rebase` merge is allowed. Never squash merge, never merge commits.
- Every commit message title MUST include its related issue number: `feat: add Foo (#42)`
- Multiple issues per commit are fine: `feat: add Bar and Baz (#43, #44)`
- Run `just preflight` before pushing.

### Quick reference

| Stage | Required? | Skill | Labels |
|-------|-----------|-------|--------|
| Grill new ideas | Always | `grill-with-docs` | — |
| Create issues | Always | — | category (`bug`/`enhancement`) + component (`rust`/`python`/`infra`) |
| Triage issues | Always | `triage` | state (`needs-triage`/`needs-info`/`ready-for-agent`/`ready-for-human`/`wontfix`) |
| Design spec | Before PR | `superpowers:brainstorming` | — |
| Draft PR | Before coding | — | — |
| Implementation plan | Before coding | `superpowers:writing-plans` | — |
| Execute plan | During coding | `superpowers:subagent-driven-development` | — |
| Code review | Before merge | `superpowers:requesting-code-review` | — |

## Architecture

### Two-layer design

**Rust layer** (`src/`): PyO3 `cdylib` crate compiled via maturin.
- `command.rs` — `RawOutput`, `CommandResult` struct
- `backend/` — `Backend` trait, `LocalBackend` (tokio::process), `SshBackend` (openssh)
- `host.rs` — `Host`, `AsyncHost`, `HostInner`, connection string parsing, `OnceLock` cache
- `helpers.rs` — `wrap_sync`, `backend_err_to_py`, `extract_args`
- `modules/` — Service, File, User, NixPackage, NixOption, SystemInfo

**Python layer** (`python/oxi_nixinfra/`): Thin shim for oxitest plugin integration.
- `__init__.py` — re-exports all types from the Rust extension
- `_plugin.py` — `oxitest_plugin()` entry point, `HostProvider` (FixtureProvider), `NixosWrapper` (ExecutionWrapper)

### Module pattern

Every module follows a three-layer pattern:
1. **Async core functions** — `pub async fn foo_impl(inner: &HostInner, ...) -> Result<T, BackendError>`
2. **Sync `#[pyclass]`** — calls `wrap_sync(&self.inner, foo_impl(...))`
3. **Async `#[pyclass]`** — clones `Arc<HostInner>`, calls `future_into_py`

### Sync + async API

- `host.service("nix-daemon").is_running()` — sync
- `host.a.service("nix-daemon").is_running()` — async (via `host.a` namespace)

### Design principle

**Shell commands produce raw output. Rust parses it.** No pipes, no `sh -c`, no `grep`. Every `execute()` call is structured args.

## Configuration

oxitest plugin settings in `pyproject.toml`:
```toml
[tool.oxitest]
plugins = ["oxi_nixinfra._plugin"]

[tool.oxitest.plugin_settings.oxi_nixinfra._plugin]
host = "ssh://dell-xps-9640"
```

Config resolution: `OXITEST_HOST` env var > pyproject.toml > `"local://"` default.

## Testing

- **Rust unit tests** (`just test-rust`): Mock `Backend` trait with `MockBackend`, test command output parsing.
- **Python integration tests** (`just test`): Run real commands on local machine. Tests marked `@oxitest.mark.nixos` auto-skip on non-NixOS.
- **CI**: Uses devenv. Runs `just check`, `just test-rust`, `just build`, `just test`. NixOS-specific tests skip automatically on Ubuntu runners.

## graphify

This project has a knowledge graph at graphify-out/. Use `graphify query`, `graphify path`, `graphify explain` when graphify-out/graph.json exists.
