#!/usr/bin/env python3
"""Offline PTY approval regression for `niuse tui`.

This is a protocol test, not a certification of a real terminal renderer. It
drives the replay provider through a write approval and checks four things:

* the configured `y allow` / `n deny` footer is visible;
* `d` no longer denies on the niuse path;
* `n` denies without touching disk;
* `y` allows and the write lands.
"""
import errno
import fcntl
import json
import os
import pty
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
from pathlib import Path


class Pane:
    def __init__(self, binary, root):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        env = dict(
            os.environ,
            HOME=str(root / "home"),
            TERM="xterm-256color",
            NO_COLOR="1",
            NIUCODE_HOME=str(root / "niucode-home"),
            NIUSE_MEMORY="off",
        )
        env.pop("NIUSE_BRIEF", None)
        self.child = subprocess.Popen(
            [
                binary,
                "tui",
                "--cwd",
                str(root / "work"),
                "--home",
                str(root / "niuse-home"),
                "--replay",
                str(root / "fixture.json"),
                "--approval",
                "ask",
                "--sandbox",
                "workspace-write",
            ],
            stdin=self.slave,
            stdout=self.slave,
            stderr=self.slave,
            env=env,
            start_new_session=True,
        )
        self.transcript = bytearray()
        self.query_tail = b""

    def pump(self, timeout):
        if not select.select([self.master], [], [], timeout)[0]:
            return
        try:
            chunk = os.read(self.master, 65536)
        except OSError as error:
            if error.errno == errno.EIO:
                return
            raise
        self.transcript.extend(chunk)
        queries = self.query_tail + chunk
        count = queries.count(b"\x1b[6n")
        if count:
            os.write(self.master, b"\x1b[24;1R" * count)
        self.query_tail = queries[-3:]

    def wait_for(self, marker, timeout=10):
        deadline = time.monotonic() + timeout
        while marker not in self.transcript:
            if self.child.poll() is not None:
                raise AssertionError(
                    f"child exited before {marker!r}\n{self.transcript[-6000:]!r}"
                )
            if time.monotonic() > deadline:
                raise AssertionError(
                    f"missing {marker!r}\n{self.transcript[-6000:]!r}"
                )
            self.pump(0.05)

    def settle(self, seconds=1.0):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            self.pump(0.05)

    def send(self, data):
        os.write(self.master, data)

    def close(self):
        if self.child.poll() is None:
            self.child.terminate()
        try:
            self.child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.child.kill()
            self.child.wait()
        os.close(self.master)
        os.close(self.slave)


def write_fixture(path, denied_answer):
    path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "turns": [
                    [
                        {
                            "type": "tool_call_delta",
                            "index": 0,
                            "id": "write-1",
                            "name": "write",
                            "arguments": json.dumps(
                                {"path": "result.txt", "content": "written\n"}
                            ),
                        }
                    ],
                    [{"type": "text_delta", "text": f"{denied_answer}\n"}],
                ],
            }
        )
    )


def run_deny(binary):
    with tempfile.TemporaryDirectory(prefix="niuse-pty-deny-") as directory:
        root = Path(directory)
        (root / "work").mkdir()
        write_fixture(root / "fixture.json", "DENIED_ANSWER")
        pane = Pane(binary, root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.send(b"write a file\r")
            pane.wait_for(b"result.txt")
            pane.wait_for(b"y allow")
            pane.wait_for(b"n deny")
            # d used to deny in the TUI pager. On the niuse path it must not.
            pane.send(b"d")
            pane.settle(1.0)
            assert b"DENIED_ANSWER" not in pane.transcript, (
                "`d` still decided the approval"
            )
            pane.send(b"n")
            pane.wait_for(b"DENIED_ANSWER")
            assert not (root / "work" / "result.txt").exists(), (
                "denied write reached disk"
            )
            print("PTY deny: configured keys shown, d ignored, n denied")
        finally:
            pane.close()


def run_allow(binary):
    with tempfile.TemporaryDirectory(prefix="niuse-pty-allow-") as directory:
        root = Path(directory)
        (root / "work").mkdir()
        write_fixture(root / "fixture.json", "ALLOWED_ANSWER")
        pane = Pane(binary, root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.send(b"write a file\r")
            pane.wait_for(b"result.txt")
            pane.wait_for(b"y allow")
            pane.send(b"y")
            pane.wait_for(b"ALLOWED_ANSWER")
            target = root / "work" / "result.txt"
            deadline = time.monotonic() + 10
            while not target.exists() and time.monotonic() < deadline:
                pane.pump(0.05)
            assert target.exists(), "allowed write did not reach disk"
            assert target.read_text() == "written\n", target.read_text()
            print("PTY allow: y allowed and wrote result.txt")
        finally:
            pane.close()


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: tui_approval.py <niuse-binary>")
    binary = sys.argv[1]
    run_deny(binary)
    run_allow(binary)


if __name__ == "__main__":
    main()
