#!/usr/bin/env python3
"""PTY regression: the after-turn memory prompt accepts or discards."""
import errno, fcntl, json, os, pty, select, struct, subprocess, sys, tempfile, termios, time
from pathlib import Path


class Pane:
    def __init__(self, binary, root):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
        env = dict(os.environ, HOME=str(root / "home"), TERM="xterm-256color", NO_COLOR="1",
                   NIUCODE_HOME=str(root / "niucode-home"))
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


def memory_text(root):
    directory = root / "niuse-home" / "memory"
    if not directory.is_dir():
        return ""
    return "\n".join(path.read_text() for path in directory.glob("*.md"))


def main():
    if len(sys.argv) not in (2, 3) or (len(sys.argv) == 3 and sys.argv[2] not in ("accept", "discard")):
        raise SystemExit("usage: tui_memory.py <niuse-binary> [accept|discard]")
    mode = sys.argv[2] if len(sys.argv) == 3 else "accept"
    binary = sys.argv[1]
    with tempfile.TemporaryDirectory(prefix="niuse-pty-memory-") as directory:
        root = Path(directory)
        (root / "work").mkdir(); (root / "niuse-home").mkdir()
        text = (
            "VISIBLE_ANSWER\n"
            "<<niuse:memory kind=preference tags=test>>\n"
            "prefer small patches\n"
            "<</niuse:memory>>\n"
        )
        (root / "fixture.json").write_text(json.dumps({
            "schema_version": 1,
            "turns": [[{"type": "text_delta", "text": text}]],
        }))
        pane = Pane(binary, root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.settle(0.3)
            os.write(pane.master, b"one\r")
            pane.wait_for(b"VISIBLE_ANSWER", 30)
            pane.wait_for(b"new memory", 10)
            choice = b"1\r" if mode == "accept" else b"3\r"
            os.write(pane.master, choice)
            pane.settle(1.0)
            stored = memory_text(root)
            if mode == "accept":
                assert "prefer small patches" in stored, stored
            else:
                assert "prefer small patches" not in stored, stored
            print(f"PTY memory {mode}: candidate {'stored' if mode == 'accept' else 'not stored'}")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
