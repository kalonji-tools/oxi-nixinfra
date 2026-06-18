"""SSH backend smoke tests for NixOS VM CI.

Run inside the VM with the Rust extension on PYTHONPATH.
Exercises every module over the SSH backend to verify end-to-end connectivity.
"""

from _oxi_nixinfra import Host

h = Host._from_config("ssh://root@localhost")

# Service module
assert h.service("sshd").is_running(), "sshd should be running"
assert h.service("sshd").exists(), "sshd unit should exist"

# File module
assert h.file("/etc/os-release").exists(), "os-release should exist"
assert h.file("/etc/os-release").is_file(), "os-release should be a regular file"

# Sysctl module
assert h.sysctl("kernel.hostname").exists(), "kernel.hostname should exist"
val = h.sysctl("kernel.hostname").value()
assert len(val) > 0, "hostname should not be empty"

# Environment module
assert h.environment().exists("PATH"), "PATH should be set"
path = h.environment().get("PATH")
assert path is not None, "PATH should have a value"

# MountPoint module
assert h.mountpoint("/").exists(), "root mount should exist"
fs = h.mountpoint("/").filesystem()
assert len(fs) > 0, "filesystem type should not be empty"

# Process module
procs = h.process().list()
assert len(procs) > 0, "should have running processes"
assert any(p["pid"] == "1" for p in procs), "PID 1 should exist"

# Socket module (parse-only — no guarantee sshd listens on 0.0.0.0:22)
sock = h.socket("tcp://0.0.0.0:22")
assert sock.protocol() == "tcp", "protocol should parse as tcp"
assert sock.port() == 22, "port should parse as 22"

print("All SSH smoke tests passed")
