# ADR-0001: Remove graphify knowledge graph tooling

**Status:** Accepted
**Date:** 2026-06-19

## Context

Graphify was integrated into oxi-nixinfra (mirroring the oxitest setup) to speed up AI-assisted codebase exploration and reduce token costs. The setup included CLAUDE.md instructions directing agents to use `graphify query`, `graphify path`, and `graphify explain`, and exclusion patterns in `prek.toml` to avoid linting graphify output.

## Decision

Remove all graphify configuration from the project. See [oxitest ADR-0001](https://github.com/kalonji-tools/oxitest/blob/main/docs/adr/0001-remove-graphify.md) for the full rationale.

## Reasons

1. **Unmeasurable value.** No metrics to verify token cost reduction or exploration speed improvement.
2. **Never used directly.** The tool operated invisibly with no way to validate correctness.
3. **Maintenance tax inherited from oxitest.** The same fragile commit workflow and stale graph problems applied here.
4. **Fully reversible.** The knowledge graph is derived from source code and can be regenerated.

## Consequences

- Agents explore the codebase using standard tools without a graph-first mandate.
- Pre-commit hooks no longer need graphify exclusion patterns.
