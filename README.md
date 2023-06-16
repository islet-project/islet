# ISLET
ISLET is a project to enable on-device confidential computing
for end users by leveraging ARMv9 CCA that is
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
- Confidential Machine Learning
- Confidential Application SDK
  - Computing API Standardization  // Multiparty CC?
- Rust-based Realm Management Monitor
