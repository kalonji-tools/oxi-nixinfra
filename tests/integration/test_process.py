"""Integration tests for Process module."""

from oxi_nixinfra import Host
from oxitest import Fixture


def test_process_list_returns_entries(host: Fixture[Host]):
    procs = host.process().list()
    assert isinstance(procs, list), "list() should return a list"
    assert len(procs) > 0, "there should be at least one running process"


def test_process_list_entry_has_keys(host: Fixture[Host]):
    procs = host.process().list()
    entry = procs[0]
    for key in ("pid", "user", "comm", "args"):
        assert key in entry, f"process entry missing '{key}' — ps output parsing broken"


def test_process_exists_init(host: Fixture[Host]):
    procs = host.process().list()
    pid_1 = [p for p in procs if p["pid"] == "1"]
    assert len(pid_1) == 1, "PID 1 should appear exactly once in process list"


def test_process_filter_by_user(host: Fixture[Host]):
    procs = host.process().filter(user="root")
    assert all(p["user"] == "root" for p in procs), (
        "filter(user='root') should only return processes owned by root"
    )


def test_process_count_nonexistent(host: Fixture[Host]):
    assert host.process().count("nonexistent_process_12345") == 0, (
        "fabricated process name should have zero count"
    )


def test_process_pids_nonexistent(host: Fixture[Host]):
    pids = host.process().pids("nonexistent_process_12345")
    assert pids == [], "fabricated process name should return empty pid list"
