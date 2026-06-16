# Apply Nix environment workarounds only when inside a Nix shell.
# Nix injects Python paths that corrupt maturin's ABI tag detection.
fix_env := if env("IN_NIX_SHELL", "") != "" {
    "unset _PYTHON_SYSCONFIGDATA_NAME PYTHONPATH &&"
} else { "" }

# maturin needs VIRTUAL_ENV to find the venv inside Nix shell.
venv_dir := env("UV_PROJECT_ENVIRONMENT", justfile_directory() / ".venv")

maturin_env := if env("IN_NIX_SHELL", "") != "" {
    "VIRTUAL_ENV=" + venv_dir
} else { "" }

# ── Color codes ────────────────────────────────────────────────────
_green := "32"
_red := "31"
_yellow := "33"
_blue := "34"

# ── Recipes ────────────────────────────────────────────────────────

# Show available recipes
default:
    @just --list

[private]
_log color msg:
    @printf '\033[{{color}}m→ %s\033[0m\n' '{{msg}}'

# Build the Rust extension
build *args: (_log _green "Building extension...")
    {{fix_env}} {{maturin_env}} maturin develop {{args}}

# Run Python integration tests (no rebuild — use `just build` first)
test *args: (_log _blue "Running integration tests...")
    PYTHONPATH=python uv run python -m oxitest tests/ {{args}}

# Run Rust unit tests
test-rust *args: (_log _blue "Running Rust tests...")
    cargo test {{args}}

# Run all static checks (format, lint, clippy)
check: (_log _blue "Running static checks...")
    ruff format --check python/ tests/
    cargo fmt --check
    ruff check python/ tests/
    cargo clippy -- -D warnings

# Full pre-push gate: clean, check, test everything
preflight: clean check test-rust build test
    @just _log {{_green}} "Preflight passed"

# Format code
fmt *args: (_log _yellow "Formatting...")
    ruff format python/ tests/
    cargo fmt {{args}}

# Remove build artifacts
clean: (_log _red "Removing build artifacts...")
    cargo clean
    rm -f python/oxi_nixinfra/_oxi_nixinfra*.so

# Check that all required tools are on PATH
health:
    #!/usr/bin/env bash
    missing=0
    for cmd in cargo maturin python3 just ruff prek; do
        if command -v "$cmd" > /dev/null 2>&1; then
            printf '  ✓ %s (%s)\n' "$cmd" "$(command -v "$cmd")"
        else
            printf '  ✗ %s NOT FOUND\n' "$cmd"
            missing=$((missing + 1))
        fi
    done
    if [ "$missing" -gt 0 ]; then
        printf '\n%d tool(s) missing\n' "$missing"
        exit 1
    else
        printf '\nAll tools available\n'
    fi
