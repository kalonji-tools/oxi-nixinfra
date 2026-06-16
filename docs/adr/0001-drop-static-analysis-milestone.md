# ADR 0001: Drop Static Analysis Milestone

**Status**: Accepted
**Date**: 2026-06-16

## Context

The v0.2.0 milestone ("Static Analysis") contained two issues:

- **#15 NixConfig**: Parse `.nix` configuration files via the `rnix` crate's AST to query NixOS option values directly, bypassing `nixos-option`.
- **#16 Drift detection**: Discovery-based scanning of a live NixOS system to find runtime state that has diverged from the current generation's declarations (undeclared services running, unmanaged files in `/etc/`, imperative users).

Both features were motivated by API ergonomics — wrapping Nix's native capabilities in oxi-nixinfra's typed Python API.

## Decision

Drop the entire v0.2.0 Static Analysis milestone. Close #15 and #16 as wontfix.

## Rationale

### Drift detection (#16)

Discovery-based drift detection would scan the system to find imperative additions (undeclared presence) and declaration violations (runtime deviations from declared state).

The target audience for oxi-nixinfra — NixOS users disciplined enough to write infrastructure tests — overlaps heavily with users of NixOS **impermanence**, which solves the lingering-state problem at a more fundamental level by wiping the root filesystem on every boot. For users without impermanence, the Venn diagram of "runs long-lived NixOS servers," "doesn't use impermanence," and "wants Python-based drift detection instead of `nixos-rebuild`" is too narrow to justify the engineering cost.

Additionally, the "violated" half of drift detection (is a declared resource in its expected runtime state?) requires knowledge of NixOS activation semantics — whether a service should be running depends on `wantedBy`, socket activation, timers, and on-demand triggers. Reimplementing that logic is a rabbit hole with no clear boundary.

### NixConfig (#15)

The `rnix` AST provides only syntactic access to `.nix` files. It cannot:

- Resolve imports (`imports = [ ./hardware-configuration.nix ]`)
- Evaluate overlays or `lib.mkMerge` / `lib.mkIf`
- Flatten the NixOS module system's merge semantics

This makes it a strict subset of `nix eval`, which gives the fully evaluated answer. Building a syntactic query layer that can't answer "what is the final value of this option?" provides little value over Nix's own tooling.

## Consequences

- oxi-nixinfra's scope stays focused on **runtime interrogation of live NixOS systems** — its existing strength and the area where Nix's native tooling has genuine gaps (especially over SSH).
- The next milestone (Extended Modules, previously v0.3.0) may be renumbered.
- Issues #15 and #16 should be closed with a reference to this ADR.
