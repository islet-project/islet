# Secure interactions with the Host

In most TEEs, interacting with the host (or the non-secure environment) is the most error-prone part
as the host can pass anything for malicious purposes.
For example, the paper named [A Tale of Two Worlds](https://people.cs.kuleuven.be/~jo.vanbulck/ccs19-tale.pdf) demonstrated that
many TEE SDKs made some mistakes while implementing such interfaces.
Also, it's not trivial to mitigate [Iago attacks](https://hovav.net/ucsd/dist/iago.pdf) that most TEEs inherently are affected by.

As Islet aims to bring the best level of security, we take those problems seriously and try to tackle them through the syntax of Rust.
This page shows the way we're doing that aspect.

## Secure host memory access

In ARM CCA, for some cases, RMM needs to map host memory and read/write something from/to that memory.
For example, when `RMI_REALM_CREATE` is invoked, RMM has to get a physical memory address where parameters are placed at
and read them from that memory.
These accesses must be securely done as insecure implementations may open things up for attackers to break ARM CCA.

We use `copy_from_host_or_ret!` and `copy_to_host_or_ret!` as a means of secure host memory access.
```rust
listen!(mainloop, rmi::REALM_CREATE, |arg, ret, rmm| {
    let rmi = rmm.rmi;
    let mm = rmm.mm;
    // key arguments
    // -- Params: the type of what the host passes
    // -- arg[1]: a physical address that points to where we should read from
    let params = copy_from_host_or_ret!(Params, arg[1], mm);
    // ... use params ...
}
```

What these two macros do is,
1. do per-struct security checks and map host memory into RMM only if it passes all checks.
2. copy from/to host memory to RMM stack memory that is bound to each CPU.
3. unmap host memory

After it gets done, we can access `params` that reside in RMM memory, not host memory.
So it's secure against concurrency-based attacks such as double-fetch attacks. 

If some additional security checks on some field value are needed (e.g., `Params.hash_algo` should be either 0 or 1),
you can do it via `validate()` in `Accessor` trait. The specified validation is called before RMM accesses host memory.
```rust
impl HostAccessor for Params {
    fn validate(&self) -> bool {
        if self.hash_algo == 0 || self.hash_algo == 1 {
            true
        } else {
            false
        }
    }
}
```

## RMI/RSI command validation

In ARM CCA, each RMI/RSI command has a different number of input/output parameters.
So we need to take special care in accessing such parameters.

To catch any mistakes regarding this in advance, Islet developers must explicitly define `Constraint` as follows.
```rust
// (1) define RMI/RSI constraints
lazy_static! {
    static ref CONSTRAINTS: BTreeMap<Command, Constraint> = {
        // This line says that RMI_DATA_CREATE has 6 input arguments and 1 output argument.
        m.insert(rmi::DATA_CREATE, Constraint::new(rmi::DATA_CREATE, 6, 1));
    }
}

// (2) check defined constraints at runtime
listen!(mainloop, rmi::DATA_CREATE, |arg, _ret, rmm| {
    // when you access arg[0], nothing happens because it doesn't cause an out-of-bound access.
    let target_pa = arg[0];
    // but, if you access arg[7], run-time panic occurs as this RMI command only has 6 arguments.
    // you can catch this error in the testing phase and fix it in advance.
    let xxx = arg[7];
}
```
