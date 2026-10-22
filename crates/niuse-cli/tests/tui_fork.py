#!/usr/bin/env python3
"""PTY regression: /fork moves niuse hooks to the fork destination."""
import errno, fcntl, json, os, pty, select, struct, subprocess, sys, tempfile, termios, time
from pathlib import Path


class Pane:
    def __init__(self, binary, root):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
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
    if len(sys.argv) != 2:
        raise SystemExit("usage: tui_fork.py <niuse-binary>")
    binary = sys.argv[1]
    with tempfile.TemporaryDirectory(prefix="niuse-pty-fork-") as directory:
        root = Path(directory)
        home = root / "niuse-home"
        (root / "work").mkdir(); home.mkdir()
        subprocess.run([binary, "--home", str(home), "plan", "add", "check the fork pointer"], check=True)
        (root / "fixture.json").write_text(json.dumps({
            "schema_version": 1,
            "turns": [[{"type": "text_delta", "text": "FORK_SOURCE_ANSWER\n"}]],
        }))
        pane = Pane(binary, root)
        try:
            pane.wait_for(b"\x1b[?2004h")
            pane.settle(0.3)
            os.write(pane.master, b"one\r")
            pane.wait_for(b"FORK_SOURCE_ANSWER")
            pane.settle(0.6)
            os.write(pane.master, b"/fork\r")
            pane.wait_for(b"fork \xc2\xb7")
            pane.settle(0.8)

            forks = [
                path for path in home.glob("*.jsonl")
                if path.name != "thread.jsonl" and "FORK_SOURCE_ANSWER" in path.read_text()
            ]
            assert forks, f"no fork destination in {list(home.glob('*.jsonl'))}"
            fork = max(forks, key=lambda path: path.stat().st_mtime)

            os.write(pane.master, b"/verify p-1 true\r")
            pane.wait_for(b"passed")
            pane.settle(0.6)

            forked = fork.read_text()
            active = (home / "thread.jsonl").read_text()
            assert '"type":"verification"' in forked, "verification missing from fork"
            assert '"step":"p-1"' in forked, forked
            assert '"type":"verification"' not in active, "verification leaked into the old thread"
            print("PTY /fork: hooks moved to the fork destination")
        finally:
            pane.close()


if __name__ == "__main__":
    main()
