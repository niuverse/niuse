#!/usr/bin/env python3
"""PTY regression: /new archives the old thread and clears its context."""
import errno, fcntl, json, os, pty, select, struct, subprocess, sys, tempfile, termios, time
from pathlib import Path


class Pane:
    def __init__(self, binary, root):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 90, 0, 0))
        env = dict(os.environ, HOME=str(root / "home"), TERM="xterm-256color", NO_COLOR="1",
                   NIUCODE_HOME=str(root / "niucode-home"), NIUSE_MEMORY="off")
        env.pop("NIUSE_BRIEF", None)
        self.child = subprocess.Popen(
            [binary, "tui", "--cwd", str(root / "work"), "--home", str(root / "niuse-home"),
             "--replay", str(root / "fixture.json"), "--approval", "never",
             "--sandbox", "read-only", "--network", "off"],
            stdin=self.slave, stdout=self.slave, stderr=self.slave, env=env, start_new_session=True)
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
            os.write(self.master, b"\x1b[24;1R" * count)
        self.tail = q[-3:]

    def wait_for(self, marker, timeout=15):
        deadline = time.monotonic() + timeout
        while marker not in self.transcript:
            if self.child.poll() is not None:
                raise AssertionError(f"child exited before {marker!r}\n{self.transcript[-5000:]!r}")
            if time.monotonic() > deadline:
                raise AssertionError(f"missing {marker!r}\n{self.transcript[-5000:]!r}")
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
    if len(sys.argv) not in (2, 3):
        raise SystemExit("usage: tui_new.py <niuse-binary> [new|reset]")
    mode = sys.argv[2] if len(sys.argv) == 3 else "new"
    if mode not in ("new", "reset"):
        raise SystemExit(f"unknown mode {mode}")
    command = b"/reset\r" if mode == "reset" else b"/new\r"
    marker = b"runtime reset" if mode == "reset" else b"new conversation"
    with tempfile.TemporaryDirectory(prefix="niuse-pty-new-") as directory:
        root = Path(directory)
        (root / "work").mkdir(); (root / "niuse-home").mkdir()
        (root / "fixture.json").write_text(json.dumps({
            "schema_version": 1,
            "turns": [
                [{"type": "text_delta", "text": "FIRST_ANSWER\n"}],
                [{"type": "text_delta", "text": "SECOND_ANSWER\n"}],
            ],
        }))
        pane = Pane(sys.argv[1], root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.settle(0.3)
            os.write(pane.master, b"one\r")
            pane.wait_for(b"FIRST_ANSWER")
            pane.settle(0.6)
            os.write(pane.master, command)
            pane.wait_for(marker)
            pane.settle(0.6)
            os.write(pane.master, b"two\r")
            pane.wait_for(b"SECOND_ANSWER")

            active = (root / "niuse-home" / "thread.jsonl").read_text()
            assert "SECOND_ANSWER" in active, active
            assert "FIRST_ANSWER" not in active, "old answer leaked into the new thread"
            archived = list((root / "niuse-home" / "threads").glob("*.jsonl"))
            assert archived, "no archived thread after /new"
            assert any("FIRST_ANSWER" in path.read_text() for path in archived)
            print(f"PTY /{mode}: old thread archived, context cleared, thread.jsonl reset")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
