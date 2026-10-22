#!/usr/bin/env python3
"""Offline PTY regression for TUI /steps and step-tagged /verify."""
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
                "never",
                "--sandbox",
                "read-only",
                "--network",
                "off",
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

    def wait_for(self, marker, timeout=15):
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

    def settle(self, seconds):
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


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: tui_plan.py <niuse-binary>")
    with tempfile.TemporaryDirectory(prefix="niuse-pty-plan-") as directory:
        root = Path(directory)
        (root / "work").mkdir()
        (root / "fixture.json").write_text(
            json.dumps({"schema_version": 1, "turns": [[{"type": "text_delta", "text": "unused\n"}]]})
        )
        pane = Pane(sys.argv[1], root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.send(b"/steps add write tests\r")
            pane.wait_for(b"added p-1")
            pane.settle(0.3)
            pane.send(b"/steps\r")
            pane.wait_for(b"no recorded verification")
            pane.settle(0.3)
            pane.send(b"/steps done 1\r")
            pane.wait_for(b"no passing verification")
            pane.settle(0.3)
            pane.send(b"/verify p-1 true\r")
            pane.wait_for(b"passed")
            pane.settle(0.5)
            pane.send(b"/steps done 1\r")
            pane.wait_for(b"done p-1")
            pane.settle(0.3)
            pane.send(b"/steps\r")
            pane.wait_for(b"no open steps")
            pane.settle(0.3)
            pane.send(b"/evidence\r")
            pane.wait_for(b"verified")

            plan = (root / "niuse-home" / "plan.md").read_text()
            assert "[x] p-1" in plan, plan
            thread = root / "niuse-home" / "thread.jsonl"
            events = [
                json.loads(line)
                for line in thread.read_text().splitlines()
                if line.strip()
            ]
            checks = [
                event
                for event in events
                if event.get("type") == "verification"
                and event.get("step") == "p-1"
                and event.get("ok") is True
            ]
            assert checks, f"no p-1 verification in thread: {events[-5:]!r}"
            print("PTY plan: /steps add → refused done → /verify p-1 → done → verified")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
