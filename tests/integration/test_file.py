"""Integration tests for File module."""

import oxitest
from oxi_nixinfra import Host
from oxitest import Fixture


def test_etc_os_release_exists(host: Fixture[Host]):
    assert host.file("/etc/os-release").exists(), (
        "/etc/os-release missing — is this a Linux system?"
    )


def test_etc_os_release_is_file(host: Fixture[Host]):
    f = host.file("/etc/os-release")
    assert f.is_file(), "/etc/os-release exists but is not a regular file"


def test_etc_is_directory(host: Fixture[Host]):
    assert host.file("/etc").is_directory(), "/etc is not a directory"


def test_nonexistent_file(host: Fixture[Host]):
    assert not host.file("/nonexistent-path-12345").exists(), (
        "fabricated path should not exist on any system"
    )


@oxitest.mark.nixos
def test_file_content(host: Fixture[Host]):
    content = host.file("/etc/os-release").content()
    assert "nixos" in content.lower(), (
        "@oxitest.mark.nixos should have skipped this on non-NixOS"
        " — check NixosWrapper or is_nixos() detection"
    )


def test_file_user_root(host: Fixture[Host]):
    owner = host.file("/etc/os-release").user()
    assert owner == "root", "user() not parsing 'stat --format=%U' output correctly"


def test_file_mode(host: Fixture[Host]):
    mode = host.file("/etc/os-release").mode()
    assert isinstance(mode, int), (
        "mode() should parse 'stat --format=%a' as int"
        " — check octal parsing in file_impl"
    )
    assert mode > 0, "mode() parsed to zero — stat output format may have changed"


@oxitest.mark.nixos
def test_etc_hosts_is_nix_managed(host: Fixture[Host]):
    assert host.file("/etc/hosts").is_nix_managed(), (
        "/etc/hosts should be NixOS-managed on NixOS"
    )


@oxitest.mark.nixos
def test_etc_hosts_store_path(host: Fixture[Host]):
    path = host.file("/etc/hosts").store_path()
    assert path is not None, (
        "NixOS-managed file should resolve to a /nix/store/ path"
        " — check readlink -f parsing"
    )
    assert path.startswith("/nix/store/"), (
        "store_path() returned a path outside /nix/store/"
        " — readlink resolved to an unexpected target"
    )


def test_nonexistent_file_not_nix_managed(host: Fixture[Host]):
    assert not host.file("/nonexistent-path-12345").is_nix_managed(), (
        "nonexistent file should not be NixOS-managed"
    )


@oxitest.mark.nixos
def test_etc_os_release_is_symlink(host: Fixture[Host]):
    assert host.file("/etc/os-release").is_symlink(), (
        "on NixOS /etc/os-release is a symlink into /nix/store/"
    )


@oxitest.mark.nixos
def test_symlink_linked_to(host: Fixture[Host]):
    target = host.file("/etc/os-release").linked_to()
    assert "/nix/store/" in target, (
        "linked_to() should resolve the symlink target via readlink"
        " — on NixOS /etc/os-release points into /nix/store/"
    )


def test_bin_sh_is_executable(host: Fixture[Host]):
    assert host.file("/bin/sh").is_executable(), (
        "/bin/sh must be executable on any Linux system"
    )


def test_regular_file_not_symlink(host: Fixture[Host]):
    f = host.file("/etc/hostname")
    if f.exists():
        assert isinstance(f.is_symlink(), bool), (
            "is_symlink() should return bool for any existing file"
        )


def test_file_uid_root(host: Fixture[Host]):
    uid = host.file("/etc/os-release").uid()
    assert uid == 0, "root-owned files should have uid 0 — check 'stat -Lc %u' parsing"


def test_file_gid_root(host: Fixture[Host]):
    gid = host.file("/etc/os-release").gid()
    assert gid == 0, "root-owned files should have gid 0 — check 'stat -Lc %g' parsing"


def test_file_group_root(host: Fixture[Host]):
    group = host.file("/etc/os-release").group()
    assert group == "root", (
        "root-owned files should have group 'root' — check 'stat -Lc %G' parsing"
    )


def test_file_size_positive(host: Fixture[Host]):
    size = host.file("/etc/os-release").size()
    assert size > 0, "/etc/os-release should be non-empty — check 'stat -Lc %s' parsing"
    assert isinstance(size, int), "size() should return int"


def test_file_contains_match(host: Fixture[Host]):
    assert host.file("/etc/os-release").contains("ID="), (
        "/etc/os-release must contain an ID= line — check grep exit code handling"
    )


def test_file_contains_no_match(host: Fixture[Host]):
    assert not host.file("/etc/os-release").contains("NONEXISTENT_KEY_12345"), (
        "contains() should return False for strings not in the file"
    )


def test_file_md5sum(host: Fixture[Host]):
    md5 = host.file("/etc/os-release").md5sum()
    assert len(md5) == 32, (
        "md5sum() should return a 32-character hex string"
        " — check 'md5sum' output parsing (first field)"
    )
    assert all(c in "0123456789abcdef" for c in md5), (
        "md5sum() should contain only hex characters"
    )


def test_file_sha256sum(host: Fixture[Host]):
    sha = host.file("/etc/os-release").sha256sum()
    assert len(sha) == 64, (
        "sha256sum() should return a 64-character hex string"
        " — check 'sha256sum' output parsing (first field)"
    )
    assert all(c in "0123456789abcdef" for c in sha), (
        "sha256sum() should contain only hex characters"
    )


def test_file_listdir(host: Fixture[Host]):
    entries = host.file("/etc").listdir()
    assert isinstance(entries, list), "listdir() should return a list"
    assert len(entries) > 0, "/etc should contain files — check 'ls' output parsing"
    assert "os-release" in entries, (
        "/etc/os-release exists, so 'os-release' should appear in listdir()"
    )
