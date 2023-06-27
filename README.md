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

![this page](https://github.com/Samsung/islet/raw/main/examples/confidential-ml/video/confidential_ml.gif)

- This video shows how ISLET achieves an end-to-end confidential machine learning with a chat-bot scenario.
- This video flows as follows.
  1. It starts with a slide that describes all components involved in this demo. All components will run on confidential computing platforms.
  2. (*feed an ML model*) The model provider feeds the ML model into the ML server. This is done through a secure channel established with the aid of the certifier framework.
  3. (*run a coding assistant*) A mobile device user asks a chat-bot application that runs on ISLET for generating a function. And then, that request is passed on to the ML server through a secure channel. Finally, the user can see the result (i.e., function).
  4. (*launch a malicious server*) This time, we launch a malicious server to show a failure case. When it attempts to join the certifier service (on the right side of the screen), it will not pass authentication as it results in a different measurement. Therefore, the malicious server cannot interact with the mobile device user in the first place.
- To download this video, click [here](https://github.com/Samsung/islet/raw/main/examples/confidential-ml/video/confidential_ml.mp4).
