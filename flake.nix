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

      # Build just the maturin wheel (no network needed)
      oxi-nixinfra-wheel = pkgs.rustPlatform.buildRustPackage {
        pname = "oxi-nixinfra";
        version = "0.2.0";
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

          # Copy wheel and project source into the VM
          vm.copy_from_host("${oxi-nixinfra-wheel}", "/tmp/wheel")
          vm.copy_from_host("${self}", "/tmp/src")

          # Extract the .so from the wheel for direct import
          vm.succeed("mkdir -p /tmp/ext && unzip -o /tmp/wheel/*.whl -d /tmp/ext 2>&1")
          vm.succeed("mv /tmp/ext/oxi_nixinfra/_oxi_nixinfra*.so /tmp/ext/ 2>&1")

          # Set up SSH key auth for root-to-localhost
          vm.succeed('ssh-keygen -t ed25519 -f /root/.ssh/id_ed25519 -N "" 2>&1')
          vm.succeed("cat /root/.ssh/id_ed25519.pub >> /root/.ssh/authorized_keys")
          vm.succeed("chmod 600 /root/.ssh/authorized_keys")
          vm.succeed("ssh-keyscan localhost >> /root/.ssh/known_hosts 2>/dev/null")

          # Run smoke tests from nix/vm_smoke_test.py
          vm.succeed("PYTHONPATH=/tmp/ext python3 /tmp/src/nix/vm_smoke_test.py 2>&1")
        '';
      };
    };
}
