# WASM REALM
The sample realm for wasm runtime.
The wasm-realm is composed with `wasmer` and `linux kernel`.
`wasmer` is statically built into rootfs for linux.

- wasmer: v3.1.4
- kernel: v5.19

## Root file system
```
├── app
│   └── hello.wasm
├── bin
│   └── wasmer
├── lib
│   └── ld-linux-aarch64.so.1
├── lib64
│   ├── libc.so.6
│   ├── libdl.so.2
│   ├── libgcc_s.so.1
│   ├── libm.so.6
│   └── libpthread.so.0
```

## Quick start on host
```sh
$ make run
Boot took 1.92 seconds
Welcome to wasm realm!

$ wasmer ./app/hello.wasm
hello, world!
```
