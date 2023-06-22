# Application Developer
Application developers are who want to develop Confidential Applications.
Confidential Application is kind of application running on Confidential Computing.

We provides `Islet SDK` which supports to build Confidential Applications.
`Islet SDK` provides Confidential Computing API (Attestation, Sealing).
You can run Confidential Applications not only on Arm FVP(arm64)
but also on Host PC(x86_64, simulated version) with `Islet SDK`.

For more information about `Islet SDK`,
please refer [this document](https://samsung.github.io/islet/components/sdk.html).

## Setting Rust environment
The first step is to prepare Rust environment.

```sh
$ ./scripts/deps/rust.sh
```

## Run the example app with SDK
You can easily explore confidential computing APIs on your x86_64 host machine.
`Islet SDK` provides code examples and the build script.

```sh
$ cd sdk
$ make run-simulated

# ISLET SDK examples: A simulated app running on x86_64
Simulated attestation operation on x86_64.
Verify Realm Signature.
== Signature Verification:
Sign Algo	 = [ES384]
Public Key	 = ["0476f988091be585ed41801aecfab858548c63057e16b
Data		 = ["846a5369676e61747572653144a1013822405901b6a70
Signature	 = ["ec4f3b28a00feabd1f58f94acb27fdc7957545409f1c9
== End of Signature Verification

...

Attestation result Ok(())
Sealing result Ok(())
```

## Example code snippet
Below is code snippet of the example.
You can refer [the whole example code](https://github.com/Samsung/islet/blob/main/sdk/examples/simulated.rs).

```rust
use islet_sdk::prelude::*;

// Attestation
let user_data = b"User data";
let report = attest(user_data)?;
let claims = verify(&report)?;
println!("Debug: {:?}", claims);

// Sealing
let plaintext = b"Plaintext";
let sealed = seal(plaintext)?;
let unsealed = unseal(&sealed)?;
assert_eq!(plaintext, &unsealed[..]);
```

