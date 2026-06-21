{
  description = "oxi-nixinfra – NixOS infrastructure testing library (oxitest plugin)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    oxitest.url = "github:kalonji-tools/oxitest";
  };

  outputs =
    {
      self,
      nixpkgs,
      oxitest,
    }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      python = pkgs.python312;
      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
      oxitest-pkg = oxitest.packages.${system}.default;

      # Build the maturin wheel from source
      oxi-nixinfra-wheel = pkgs.rustPlatform.buildRustPackage {
        pname = "oxi-nixinfra-wheel";
        inherit version;
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [
          pkgs.maturin
          python
        ];
        buildPhase = ''
          maturin build --release --interpreter ${python}/bin/python3
        '';
        installPhase = ''
          mkdir -p $out
          cp target/wheels/*.whl $out/
        '';
        doCheck = false;
      };

      # Install the wheel into a Python package with oxitest as a dependency
      oxi-nixinfra = python.pkgs.buildPythonPackage {
        pname = "oxi-nixinfra";
        inherit version;
        format = "other";
        dontUnpack = true;
        dontBuild = true;
        nativeBuildInputs = [ python.pkgs.installer ];
        propagatedBuildInputs = [ oxitest-pkg ];
        installPhase = ''
          ${python}/bin/python3 -m installer --destdir="$out" --prefix="" ${oxi-nixinfra-wheel}/*.whl
        '';
        pythonImportsCheck = [ "oxi_nixinfra" ];
      };
    in
    {
      packages.${system}.default = oxi-nixinfra;

      checks.${system}.integration = pkgs.testers.nixosTest {
        name = "oxi-nixinfra-integration";

        nodes.vm = { pkgs, ... }: {
          imports = [ ./nix/test-vm.nix ];

          virtualisation = {
            memorySize = 2048;
            cores = 2;
            writableStoreUseTmpfs = false;
          };
        };

        testScript =
          let
            sitePackages = "${oxi-nixinfra}/${python.sitePackages}";
            oxitestSitePackages = "${oxitest-pkg}/${python.sitePackages}";
          in
          ''
            vm.start()
            vm.wait_for_unit("sshd")
            vm.wait_for_open_port(22)

            # nix-daemon is socket-activated — start it explicitly so tests see it as "active"
            vm.succeed("systemctl start nix-daemon")

            # Copy project source into the VM
            vm.copy_from_host("${self}", "/tmp/src")

            # nixosTest VMs don't go through nixos-rebuild, so no generation links exist.
            # Create one to match what every real NixOS install has.
            vm.succeed("ln -s /run/current-system /nix/var/nix/profiles/system-1-link")

            # Set up SSH key auth for root-to-localhost
            vm.succeed('ssh-keygen -t ed25519 -f /root/.ssh/id_ed25519 -N "" 2>&1')
            vm.succeed("cat /root/.ssh/id_ed25519.pub >> /root/.ssh/authorized_keys")
            vm.succeed("chmod 600 /root/.ssh/authorized_keys")
            vm.succeed("ssh-keyscan localhost >> /root/.ssh/known_hosts 2>/dev/null")

            # Run 1: local backend
            vm.succeed("cd /tmp/src && PYTHONPATH=${sitePackages}:${oxitestSitePackages} python3 -m oxitest run tests/ 2>&1")

            # Run 2: SSH backend
            vm.succeed("cd /tmp/src && PYTHONPATH=${sitePackages}:${oxitestSitePackages} OXITEST_HOST=ssh://root@localhost python3 -m oxitest run tests/ 2>&1")
          '';
      };
    };
}
