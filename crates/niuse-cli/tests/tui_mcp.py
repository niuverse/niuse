#!/usr/bin/env python3
"""Offline PTY regression: niuse tui mounts read-only MCP and dispatches a tool.

The replay provider calls the notes pack. The test checks the startup mount line,
the visible answer, and the tool result in the append-only thread.
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
        env.pop("NIUSE_MCP_CONFIG", None)
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


def write_manifest(root, python, notes, server):
    (root / "niucode-home").mkdir(parents=True, exist_ok=True)
    (root / "niucode-home" / "mcp.toml").write_text(
        f"""[[server]]
name = "notes"
command = {json.dumps(python)}
args = [{json.dumps(str(server))}]
read_only = true

[server.env]
NOTES_DIR = {json.dumps(str(notes))}
PATH = {json.dumps(os.environ.get("PATH", "/usr/bin:/bin"))}
"""
    )


def write_fixture(path):
    path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "turns": [
                    [
                        {
                            "type": "tool_call_delta",
                            "index": 0,
                            "id": "mcp-1",
                            "name": "mcp__notes__search_notes",
                            "arguments": json.dumps({"query": "rust"}),
                        }
                    ],
                    [{"type": "text_delta", "text": "MCP_TOOL_OK\n"}],
                ],
            }
        )
    )


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: tui_mcp.py <niuse-binary>")
    binary = sys.argv[1]
    with tempfile.TemporaryDirectory(prefix="niuse-pty-mcp-") as directory:
        root = Path(directory)
        (root / "work").mkdir()
        notes = root / "notes"
        notes.mkdir()
        (notes / "note.md").write_text("Rust note for MCP\n")
        server = (
            Path(__file__).resolve().parent.parent.parent.parent
            / "packs"
            / "notes"
            / "server.py"
        )
        write_manifest(root, sys.executable, notes, server)
        write_fixture(root / "fixture.json")

        pane = Pane(binary, root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.send(b"search my notes\r")
            pane.wait_for(b"mcp: notes")
            pane.wait_for(b"MCP_TOOL_OK")
            thread = root / "niuse-home" / "thread.jsonl"
            deadline = time.monotonic() + 10
            while not thread.exists() and time.monotonic() < deadline:
                pane.pump(0.05)
            assert thread.exists(), "thread.jsonl was not created"
            events = [
                json.loads(line)
                for line in thread.read_text().splitlines()
                if line.strip()
            ]
            calls = [
                event
                for event in events
                if event.get("type") == "assistant_message"
                and any(
                    call.get("name") == "mcp__notes__search_notes"
                    for call in event.get("tool_calls", [])
                )
            ]
            results = [
                event
                for event in events
                if event.get("type") == "tool_result"
                and "note.md" in event.get("output", "")
            ]
            assert calls, f"no MCP tool call in thread: {events[-5:]!r}"
            assert results, f"no notes tool result in thread: {events[-5:]!r}"
            assert results[0].get("ok") is True
            print("PTY MCP: mounted, called mcp__notes__search_notes, got note.md")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
