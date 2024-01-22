# Command Line Interface
Islet provides Command Line Interface (CLI) which can explore CCA operations.
CLI supports both x86_64 (simulated) and aarch64.

You can explore Attestation like below
```sh
$ cd cli
$ make x86_64
$ ./islet attest --output=./report
$ ./islet verify --input=./report

== Signature Verification:
Sign Algo        = [ES384]
Public Key       = ["0476f988091be585ed41801aecfab858...]
Data             = ["846a5369676e61747572653144a10138...]
Signature        = ["ec4f3b28a00feabd1f58f94acb27fdc7...]
== End of Signature Verification

== Realm Token cose:
Protected header               = Header { alg: Some(Assigned(ES
Unprotected header             = Header { alg: None, crit: [],
Signature                      = [ec4f3b28a00feabd1f58f94acb27f
== End of Realm Token cose

== Realm Token:
Realm challenge                (#10) = [abababababababababababa
Realm personalization value    (#44235) = [00000000000000000000
Realm hash algo id             (#44236) = "sha-256"
Realm public key hash algo id  (#44240) = "sha-256"
Realm signing public key       (#44237) = [0476f988091be585ed41
Realm initial measurement      (#44238) = [6190eb90b293886c172e
```

