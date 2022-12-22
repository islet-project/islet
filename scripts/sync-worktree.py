#!/usr/bin/env python3

import glob
import os
import subprocess
import sys

from config import THIRD_PARTY

def sync():
    third_parties = []
    for path in glob.glob(os.path.join(THIRD_PARTY, "*/")):
        name = os.path.basename(path.rstrip("/"))
        process = subprocess.run(
            ['git', 'worktree', 'add', '--guess-remote', f'third-party/{name}'],
            stdout=subprocess.PIPE, universal_newlines=True)
        if process.returncode != 0:
            print("You might already sync worktree. "
                  f"Or remove local branch [{name}] "
                  "to fetch it from remote.")

if __name__ == "__main__":
    sync()
