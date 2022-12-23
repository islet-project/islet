#!/usr/bin/env python3

import glob
import os
import subprocess
import sys

from config import ROOT, THIRD_PARTY

def run(cmd, cwd):
    return subprocess.run(cmd, cwd=cwd,
            stderr=subprocess.STDOUT,
            stdout=subprocess.PIPE,
            universal_newlines=True,
            check=False)

def branch_name(path):
    proc = run(['git', 'rev-parse', '--abbrev-ref', 'HEAD'], cwd=path)
    return proc.stdout.strip()

def sync():
    third_parties = []
    for path in glob.glob(os.path.join(THIRD_PARTY, "*/")):
        name = os.path.basename(path.rstrip("/"))
        run(['git', 'rm', '--cached', f'third-party/{name}'], cwd=ROOT)

        proc = run(['git', 'worktree', 'add', '--guess-remote', f'third-party/{name}'], cwd=ROOT)
        if proc.returncode != 0 and name != branch_name(path):
            print(f"Please remove local branch [{name}] to fetch it from remote.")
            sys.exit(1)

if __name__ == "__main__":
    sync()
