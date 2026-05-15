# Secure interactions with the Host

In most TEEs, interacting with the host (or the non-secure environment) is the most error-prone part
as the host can pass anything for malicious purposes.
For example, the paper [A Tale of Two Worlds](https://people.cs.kuleuven.be/~jo.vanbulck/ccs19-tale.pdf) demonstrated that
many TEE SDKs made mistakes when implementing such interfaces.
It is also not trivial to mitigate [Iago attacks](https://hovav.net/ucsd/dist/iago.pdf), which affect most TEEs by design.

As Islet aims to provide the best level of security, we take these problems seriously and try to tackle them through Rust's syntax.
This page describes how we address these issues.

## Secure host memory access

In ARM CCA, for some cases, RMM needs to map host memory and read data from or write data to that memory.
For example, when `RMI_REALM_CREATE` is invoked, RMM must read parameters from a physical memory address provided by the host.
These accesses must be performed securely, as insecure implementations may allow attackers to compromise ARM CCA.

To improve the security of host memory access, dedicated helper functions are defined in `rmm/src/host.rs`:

- `copy_from()` to copy raw data from a memory pointer into a newly instantiated data structure that implements the `SafetyChecked` and `SafetyAssured` traits.
- `copy_to_obj()` to copy raw data from a memory pointer into an existing data structure that implements the `SafetyChecked` and `SafetyAssured` traits.
- `copy_to_ptr()` to copy a data structure that implements the `SafetyChecked` and `SafetyAssured` traits into a memory region pointed to by a raw pointer.

Here is an example of `copy_from()` usage in the `REALM_CREATE` handler:
```rust
listen!(mainloop, rmi::REALM_CREATE, |arg, ret, rmm| {
    // ...
    let params_ptr = arg[1];
    // key arguments
    // -- Params: the type of what the host passes
    // -- arg[1]: a physical address that points to where we should read from
    let params = host::copy_from::<Params>(params_ptr).ok_or(Error::RmiErrorInput)?;
    // ... use params ...
}
```

What these functions do:
1. Perform per-struct security checks, such as checking the granule physical address and its state
2. Copy data between host memory and RMM memory

Once this is done, we can access `params`, which resides in RMM memory, not host memory.
This makes the access secure against concurrency-based attacks such as double-fetch attacks.

If additional security checks on field values are needed, RMM structures such as `Params` and `Run` implement an additional `verify_compliance()` function (e.g., `rmm/src/rmi/realm/params.rs`) that performs checks before the structure is used.

## RMI/RSI command validation

In ARM CCA, each RMI/RSI command has a different number of input/output parameters.
Therefore, special care is needed when accessing these parameters.

To catch such mistakes in advance, Islet developers must explicitly define a `Constraint` for each command.

For RMI calls, constraints are defined in `rmm/src/rmi/constraint.rs`:

```rust
fn pick(cmd: Command) -> Option<Constraint> {
    let constraint = match cmd {
        rmi::VERSION => Constraint::new(rmi::VERSION, 2, 3),
        rmi::GRANULE_DELEGATE => Constraint::new(rmi::GRANULE_DELEGATE, 2, 1),
        rmi::GRANULE_UNDELEGATE => Constraint::new(rmi::GRANULE_UNDELEGATE, 2, 1),
        rmi::DATA_CREATE => Constraint::new(rmi::DATA_CREATE, 6, 1),
        ...
    }
}
```

These constraints are verified by the `validate()` function defined in `rmm/src/rmi/constraint.rs`. The `validate()` function is used by the main RMI command dispatching loop defined in `islet/rmm/src/event/mainloop.rs`.

```rust
// (2) check defined constraints at runtime
listen!(mainloop, rmi::DATA_CREATE, |arg, _ret, rmm| {
    // when you access arg[0], nothing happens because it doesn't cause an out-of-bound access.
    let target_pa = arg[0];
    // but, if you access arg[7], run-time panic occurs as this RMI command only has 6 arguments.
    // you can catch this error in the testing phase and fix it in advance.
    let xxx = arg[7];
}
```

For RSI calls, constraints are defined in `islet/rmm/src/rsi/constraint.rs`. The analogous `validate()` function is called during handling of RSI commands and is defined in `rmm/src/rsi/constraint.rs`.
