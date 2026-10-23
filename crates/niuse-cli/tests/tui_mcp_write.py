#!/usr/bin/env python3
"""PTY regression: TUI mutating-MCP approval preview and outcome."""
import errno, fcntl, json, os, pty, select, struct, subprocess, sys, tempfile, termios, time
from pathlib import Path


class Pane:
    def __init__(self, binary, root):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
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
                binary, "tui",
                "--cwd", str(root / "work"),
                "--home", str(root / "niuse-home"),
                "--replay", str(root / "fixture.json"),
                "--approval", "ask",
                "--allow-mcp-write",
                "--sandbox", "read-only",
                "--network", "off",
            ],
            stdin=self.slave, stdout=self.slave, stderr=self.slave, env=env,
            start_new_session=True,
        )
        self.transcript = bytearray(); self.tail = b""

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
        q = self.tail + chunk
        count = q.count(b"\x1b[6n")
        if count:
            os.write(self.master, b"\x1b[30;1R" * count)
        self.tail = q[-3:]

    def wait_for(self, marker, timeout=20):
        deadline = time.monotonic() + timeout
        while marker not in self.transcript:
            if self.child.poll() is not None:
                raise AssertionError(f"child exited before {marker!r}\n{self.transcript[-6000:]!r}")
            if time.monotonic() > deadline:
                raise AssertionError(f"missing {marker!r}\n{self.transcript[-6000:]!r}")
            self.pump(0.05)

    def settle(self, seconds):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            self.pump(0.05)

    def close(self):
        if self.child.poll() is None:
            self.child.terminate()
        try:
            self.child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.child.kill(); self.child.wait()
        os.close(self.master); os.close(self.slave)


def main():
    if len(sys.argv) not in (2, 3) or (len(sys.argv) == 3 and sys.argv[2] not in ("allow", "deny")):
        raise SystemExit("usage: tui_mcp_write.py <niuse-binary> [allow|deny]")
    mode = sys.argv[2] if len(sys.argv) == 3 else "deny"
    with tempfile.TemporaryDirectory(prefix="niuse-pty-mcp-write-") as directory:
        root = Path(directory)
        (root / "work").mkdir()
        (root / "niucode-home").mkdir()
        notes = root / "notes"; notes.mkdir()
        server = Path(__file__).resolve().parent.parent.parent.parent / "packs" / "notes" / "server.py"
        (root / "niucode-home" / "mcp.toml").write_text(f'''[[server]]
name = "notes"
command = {json.dumps(sys.executable)}
args = [{json.dumps(str(server))}]
read_only = false
write_tools = ["create_note"]

[server.env]
NOTES_DIR = {json.dumps(str(notes))}
NOTES_WRITE = "1"
PATH = {json.dumps(os.environ.get("PATH", "/usr/bin:/bin"))}
''')
        answer = "WRITE_ALLOWED" if mode == "allow" else "WRITE_DENIED"
        (root / "fixture.json").write_text(json.dumps({
            "schema_version": 1,
            "turns": [
                [{"type": "tool_call_delta", "index": 0, "id": "mcp-write-1",
                  "name": "mcp__notes__create_note",
                  "arguments": json.dumps({"title": "approval test", "text": "approved body"})}],
                [{"type": "text_delta", "text": answer + "\n"}],
            ],
        }))
        pane = Pane(sys.argv[1], root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.pump(0.2)
            os.write(pane.master, b"write a note\r")
            pane.wait_for(b"mcp: notes")
            pane.wait_for(b"mutating MCP")
            pane.wait_for(b"mcp__notes__create_note")
            pane.wait_for(b"Arguments:")
            os.write(pane.master, b"y" if mode == "allow" else b"n")
            pane.wait_for(answer.encode())
            note = notes / "approval-test.md"
            if mode == "allow":
                assert note.exists(), "approved write did not create the note"
                assert "approved body" in note.read_text()
            else:
                assert not note.exists(), "denied write created the note"
            pane.pump(0.2)
            thread = root / "niuse-home" / "thread.jsonl"
            deadline = time.monotonic() + 5
            while not thread.exists() and time.monotonic() < deadline:
                pane.pump(0.05)
            events = [
                json.loads(line)
                for line in thread.read_text().splitlines()
                if line.strip()
            ]
            mounts = [event for event in events if event.get("type") == "mcp_mount"]
            assert mounts, f"no mcp_mount event: {events[-5:]!r}"
            mount = mounts[0]
            assert mount.get("server") == "notes"
            assert mount.get("write_tools") == ["mcp__notes__create_note"]
            assert len(mount.get("config_hash", "")) == 16, mount
            pane.settle(0.3)
            os.write(pane.master, b"/evidence\r")
            pane.wait_for(b"writes:")
            pane.wait_for(b"mcp mount(s)")
            print(f"PTY mutating MCP {mode}: preview, outcome, mount audit, evidence correct")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
