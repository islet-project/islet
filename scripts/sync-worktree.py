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

def commit(path):
    proc = run(['git', 'rev-parse', '--short', 'HEAD'], cwd=path)
    return proc.stdout.strip()

def check_exist(project, info):
    proc = run(['git', 'worktree', 'list'], cwd=ROOT)
    for line in proc.stdout.splitlines():
        if line.find(info["commit"]) > 0:
            return True

    return False

def add_worktree(project, info):
    branch = info["branch"]
    print(f"[+] Sync worktree to third-party/{project}: branch[{branch}]")

    proc = run(['git', 'worktree', 'add', '--guess-remote', f'third-party/{branch}'], cwd=ROOT)
    print(proc)
    if proc.returncode != 0 and branch != branch_name(os.path.join(THIRD_PARTY, project)):
        print(f"[-] Please remove local branch [{branch}] to fetch it from remote.")
        sys.exit(1)

    run(['git', 'worktree', 'move', branch, f'third-party/{project}'], cwd=ROOT)
 
    if info["commit"] != commit(os.path.join(THIRD_PARTY, project)):
        print(f"[-] Mismatched [{project}] commit: {commit}")
        sys.exit(1)

if __name__ == "__main__":
    tree = toml.load(os.path.join(THIRD_PARTY, "worktree.toml"))

    for project, info in tree.items():
        if check_exist(project, info):
            print(f"[!] Already added project [{project}]")
            continue

        if len(sys.argv) == 1:
            add_worktree(project, info)
        elif len(sys.argv) == 2 and sys.argv[1] == project:
            add_worktree(project, info)
