# ISLET
ISLET is a project of Samsung Research that extends confidential computing to user devices.
We enable on-device confidential computing for end users by leveraging ARMv9 CCA that is
the newly emerging confidential computing hardware on ARM devices.

Using the hardware support, ISLET enables a Trusted Execution Environment (TEE)
on user’s devices within which users can securely process, store, communicate
and manage their private data. The protection provided by
ISLET applies not only to data-at-rest but also to data-in-use
even in the presence of malicious privileged software on devices.

We develop components enabling Realm Virtual Machines (VMs),
which are secure VM-level TEE provided by ARMv9 CCA.
To manage Realm VMs, Realm Management Monitor (RMM)
is needed to be running at EL2 in the Realm world.
ISLET provides the implementation of RMM that is written in Rust. 

## Feature Overview
- Rust-based Realm Management Monitor
- Confidential Computing API Standardization
- Confidential Machine Learning

## Overall Architecture
We contribute mainly to `Islet RMM`, `Islet SDK`.

```
 << Realm World >>   << Normal World >>
┌──────────────────┐┌──────────────────┐
│ Confidential App ││                  │
├──────────────────┤│                  │
│ Islet SDK        ││ Linux            │
├──────────────────┤│                  │
│ Kernel           ││                  │
├──────────────────┤├──────────────────┤
│ Islet RMM        ││ KVM              │
└──────────────────┘└──────────────────┘
┌──────────────────────────────────────┐
│ TF-A (EL3 Firmware)                  │
└──────────────────────────────────────┘
┌──────────────────────────────────────┐┌──────────────────┐
│ Arm Fixed Virtual Platforms (arm64)  ││ ISLET CLI        │
└──────────────────────────────────────┘└──────────────────┘
┌──────────────────────────────────────────────────────────┐
│ Host (x86_64)                                            │
└──────────────────────────────────────────────────────────┘

Where :
  RMM - Realm Management Monitor
  KVM - Kernel-based Virtual Machine
```

---

For more information, please visit our [developer site](https://samsung.github.io/islet/).

## A demo video (Confidential ML)

![this page](./examples/confidential-ml/video/confidential_ml.gif).

See [this page](./examples/confidential-ml/README.md) to get details about this demo.
