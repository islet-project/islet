# Third-party projects

ISLET uses several third-party projects for realm, normal-world and testing.
Third-party projects are managed using `worktree` 
which means they are forked from upstream to the branch of ISLET repo.

You can check below after `scripts/init.sh` or `scripts/sync-worktree.sh`.

```
~/islet $ tree -L 2
.
├── assets # submodule
├── third-party
│   ├── android-kernel # worktree
│   ├── gki-build      # worktree
│   ├── nw-linux       # worktree
│   ├── optee-build    # worktree
│   ├── realm-linux    # worktree
│   ├── tf-a           # worktree
│   ├── tf-a-tests     # worktree
```

## How to add patches to third-parties
1. Fork the third-party branch to another branch (ex. 3rd-nw-linux-XXXXXX)
2. Add patches and test
3. Update `third-party/worktree.toml` and make pull-request
