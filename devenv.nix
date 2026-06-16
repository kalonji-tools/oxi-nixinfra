{ pkgs, ... }:

let
  python = pkgs.python312;
  pythonEnv = python.withPackages (ps: [ ps.pip ]);
in
{
  languages.rust.enable = true;

  languages.python = {
    enable = true;
    package = pythonEnv;
    uv = {
      enable = true;
      sync.enable = true;
    };
  };

  packages = with pkgs; [
    maturin
    just
    ruff
    prek
  ];

  env = {
    RUST_BACKTRACE = "1";
    PYO3_PYTHON = "${pythonEnv}/bin/python3";
  };

  tasks."oxi-nixinfra:install-hooks" = {
    exec = ''
      if git rev-parse --git-dir > /dev/null 2>&1; then
        git config --unset core.hooksPath 2>/dev/null || true
        prek install --quiet
        HOOKS_DIR="$(git rev-parse --git-common-dir)/hooks"
        git config core.hooksPath "$HOOKS_DIR"
      fi
    '';
    before = [ "devenv:enterShell" ];
  };

  enterShell = ''
    export PATH="$HOME/.local/bin:$UV_PROJECT_ENVIRONMENT/bin:$PATH"

    # Symlink .venv -> devenv venv so tools with hardcoded .venv paths work
    if [ "$UV_PROJECT_ENVIRONMENT" != "$PWD/.venv" ] && [ ! -L .venv ]; then
      ln -snf "$UV_PROJECT_ENVIRONMENT" .venv
    fi

    # Build oxitest .so if missing or stale, then install editable
    OXITEST_DIR="../oxitest.main"
    if [ -d "$OXITEST_DIR/src" ]; then
      PY_VER=$(python3 -c 'import sys; print(f"cpython-{sys.version_info[0]}{sys.version_info[1]}")')
      OXITEST_SO=$(find "$OXITEST_DIR/python/oxitest" -name "_oxitest.$PY_VER*.so" 2>/dev/null | head -1)
      if [ -z "$OXITEST_SO" ]; then
        echo "Building oxitest extension..."
        (cd "$OXITEST_DIR" && just build)
      elif [ -n "$(find "$OXITEST_DIR/src" -name '*.rs' -newer "$OXITEST_SO" 2>/dev/null | head -1)" ]; then
        echo "oxitest Rust source changed, rebuilding..."
        (cd "$OXITEST_DIR" && just build)
      fi
      # Install editable oxitest into this project's venv
      uv pip install -e "$OXITEST_DIR" --quiet 2>/dev/null || true
    fi

    # Build oxi-nixinfra .so if missing or stale
    PY_VER=$(python3 -c 'import sys; print(f"cpython-{sys.version_info[0]}{sys.version_info[1]}")')
    SO=$(find python/oxi_nixinfra -name "_oxi_nixinfra.$PY_VER*.so" 2>/dev/null | head -1)
    if [ -z "$SO" ]; then
      echo "Building oxi-nixinfra extension..."
      just build
    elif [ -n "$(find src -name '*.rs' -newer "$SO" 2>/dev/null | head -1)" ] || \
         [ Cargo.toml -nt "$SO" ]; then
      echo "Rust source changed, rebuilding extension..."
      just build
    fi

    just health
  '';
}
