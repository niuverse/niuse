#!/usr/bin/env python3
"""PTY regression: /brief, /report, and /steps compose in one TUI session."""
import errno, fcntl, json, os, pty, select, struct, subprocess, sys, tempfile, termios, time
from pathlib import Path


class Pane:
    def __init__(self, binary, root):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 110, 0, 0))
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
            os.write(self.master, b"\x1b[30;1R" * count)
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
    if len(sys.argv) != 2:
        raise SystemExit("usage: tui_commands.py <niuse-binary>")
    binary = sys.argv[1]
    with tempfile.TemporaryDirectory(prefix="niuse-pty-commands-") as directory:
        root = Path(directory)
        home = root / "niuse-home"
        work = root / "work"
        (work / "docs").mkdir(parents=True); home.mkdir()
        subprocess.run([binary, "--home", str(home), "plan", "add", "check brief"], check=True)
        subprocess.run([binary, "--home", str(home), "goals", "add", "ship the stage"], check=True)
        subprocess.run(
            [binary, "--home", str(home), "loops", "add", "--due", "2026-10-22",
             "finish the brief check"],
            check=True,
        )
        (work / "docs" / "dogfood.md").write_text(
            "| date | turns | proposed | accepted | note |\n"
            "|---|---|---|---|---|\n"
            "| 2026-10-22 | 2 | 1 | 1 | cited m-abcd |\n"
        )
        (root / "fixture.json").write_text(json.dumps({"schema_version": 1, "turns": []}))
        pane = Pane(binary, root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.settle(0.3)
            os.write(pane.master, b"/brief\r")
            pane.wait_for(b"niuse brief")
            pane.wait_for(b"l-")
            pane.settle(0.4)
            os.write(pane.master, b"/report\r")
            pane.wait_for(b"# niuse weekly report")
            pane.wait_for(b"Dogfood:")
            pane.settle(0.4)
            os.write(pane.master, b"/steps\r")
            pane.wait_for(b"p-1 check brief")
            print("PTY /brief /report /steps: all composed in one session")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
