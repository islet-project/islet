#!/usr/bin/env python3

import glob
import os
import subprocess
import sys
import toml

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

def add_worktree(project, branch):
    print(f"[!] Sync worktree to third-party/{project}: branch[{branch}]")
    run(['git', 'rm', '--cached', f'third-party/{project}'], cwd=ROOT)

    proc = run(['git', 'worktree', 'add', '--guess-remote', f'third-party/{branch}'], cwd=ROOT)
    if proc.returncode != 0 and branch != branch_name(os.path.join(THIRD_PARTY, project)):
        print(f"[!] Please remove local branch [{branch}] to fetch it from remote.")
        sys.exit(1)

    run(['git', 'worktree', 'move', branch, f'third-party/{project}'], cwd=ROOT)

if __name__ == "__main__":
    tree = toml.load(os.path.join(THIRD_PARTY, "worktree.toml"))
    for project, info in tree.items():
        add_worktree(project, info["branch"])
