# Minimal NixOS VM for integration testing.
# Root with empty password — ephemeral, destroyed after test run.
{ pkgs, ... }:
{
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "yes";
      PermitEmptyPasswords = "yes";
    };
  };

  users.users.root.password = "";

  networking.firewall.allowedTCPPorts = [ 22 ];

  environment.systemPackages = with pkgs; [
    # Packages needed by integration tests
    procps        # ps
    iproute2      # ss
    util-linux    # findmnt
    coreutils     # printenv, readlink, stat, etc.
    unzip         # extract .so from wheel
    python312     # runtime for smoke tests
    openssh       # SSH client (needed by the SSH backend to connect to localhost)
  ];

  system.stateVersion = "25.05";
}
