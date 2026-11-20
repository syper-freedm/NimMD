"""Проверяет отслеживаемые и неигнорируемые неотслеживаемые файлы."""

import os
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory


def git_paths(*args):
    output = subprocess.check_output(["git", "ls-files", "-z", *args])
    return {Path(os.fsdecode(path)) for path in output.split(b"\0") if path}


def main():
    files = git_paths("--cached", "--others", "--exclude-standard")
    with TemporaryDirectory(prefix="nimmd-gitleaks-") as directory:
        snapshot = Path(directory)
        for path in files:
            if path.is_symlink() or not path.is_file():
                continue
            destination = snapshot / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
        return subprocess.run(
            ["gitleaks", "dir", "--redact", "--no-banner", str(snapshot)], check=False
        ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
