{
  description = "oxi-nixinfra CI checks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      python = pkgs.python312;

      # Pre-built oxitest wheel from PyPI (pinned version + hash)
      # name preserves the wheel filename — pip requires it for version/platform parsing
      oxitest-wheel = pkgs.fetchurl {
        name = "oxitest-1.0.0b1-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl";
        url = "https://files.pythonhosted.org/packages/9e/c6/3375076d99890da48c702d513a37ab592512c5e098eee41bbbf07493075a/oxitest-1.0.0b1-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl";
        sha256 = "1sfqaa1nb52wmlqwg66h1jxr1h2613s2hq7vc2qm1dg74waidaaf";
      };

      # Build just the maturin wheel (no network needed)
      oxi-nixinfra-wheel = pkgs.rustPlatform.buildRustPackage {
        pname = "oxi-nixinfra";
        version = "0.4.0";  # oxi-nixinfra
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [ pkgs.maturin python ];
        buildPhase = ''
          maturin build --release --interpreter ${python}/bin/python3
        '';
        installPhase = ''
          mkdir -p $out
          cp target/wheels/*.whl $out/
        '';
        doCheck = false;
      };
    in
    {
      checks.${system}.integration = pkgs.testers.nixosTest {
        name = "oxi-nixinfra-integration";

        nodes.vm = { pkgs, ... }: {
          imports = [ ./nix/test-vm.nix ];

          virtualisation = {
            memorySize = 2048;
            cores = 2;
            # Share host nix store with VM for faster access
            writableStoreUseTmpfs = false;
          };
        };

        testScript = ''
          vm.start()
          vm.wait_for_unit("sshd")
          vm.wait_for_open_port(22)

          # nix-daemon is socket-activated — start it explicitly so tests see it as "active"
          vm.succeed("systemctl start nix-daemon")

          # Copy wheels and project source into the VM
          vm.succeed("mkdir -p /tmp/wheels")
          vm.copy_from_host("${oxitest-wheel}", "/tmp/wheels/oxitest.whl")
          vm.copy_from_host("${oxi-nixinfra-wheel}", "/tmp/oxi-nixinfra-wheel")
          vm.copy_from_host("${self}", "/tmp/src")

          # Rename oxitest wheel to its original filename (pip requires it for parsing)
          vm.succeed("mv /tmp/wheels/oxitest.whl /tmp/wheels/${oxitest-wheel.name}")

          # Install both wheels to a writable directory (Nix store is read-only)
          vm.succeed("pip install --no-deps --break-system-packages --target /tmp/site-packages /tmp/wheels/*.whl /tmp/oxi-nixinfra-wheel/*.whl 2>&1")

          # nixosTest VMs don't go through nixos-rebuild, so no generation links exist.
          # Create one to match what every real NixOS install has.
          vm.succeed("ln -s /run/current-system /nix/var/nix/profiles/system-1-link")

          # Set up SSH key auth for root-to-localhost
          vm.succeed('ssh-keygen -t ed25519 -f /root/.ssh/id_ed25519 -N "" 2>&1')
          vm.succeed("cat /root/.ssh/id_ed25519.pub >> /root/.ssh/authorized_keys")
          vm.succeed("chmod 600 /root/.ssh/authorized_keys")
          vm.succeed("ssh-keyscan localhost >> /root/.ssh/known_hosts 2>/dev/null")

          # Run 1: local backend
          vm.succeed("cd /tmp/src && PYTHONPATH=/tmp/site-packages python3 -m oxitest run tests/ 2>&1")

          # Run 2: SSH backend
          vm.succeed("cd /tmp/src && PYTHONPATH=/tmp/site-packages OXITEST_HOST=ssh://root@localhost python3 -m oxitest run tests/ 2>&1")
        '';
      };
    };
}
