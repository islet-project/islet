# How to Debug with TRACE32

A script for debugging rmm using TRACE32 is provided at debug/t32.cmm.

## Instructions
1. Install Trace 32
2. Edit config.t32 on the installed path and uncomment the line below
```
- ;PBI=CADI
+ PBI=CADI
```
3. Run with the --debug option
```bash
./scripts/fvp --debug -ro -nw {linux|tf-a-tests| -vm {linux|tftf}
```

# Debugging with Realm
see documentation [Debug Realm](debug_realm.md)
