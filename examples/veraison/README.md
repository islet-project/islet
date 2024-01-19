# Introduction

The process consists of several parts:

* provisioning
* gathering measurements
* feeding measurements to veraison and realm verifier
* running realm for verification purposes
* running verification services (veraison/realm verifier)
* verification itself

It is best and even sometimes required that all the required repos are placed in
one directory. I'll call it `CCA` and it will be referred throughout this file.

The following repos will be used:

* Islet: https://github.com/islet-project/islet that provides:

	* the whole SW/FW stack and scripts for running the emulated environment under the FVP
	* Islet HES https://github.com/islet-project/islet/tree/main/hes
	* kvmtool-rim-measurer from https://github.com/islet-project/islet/tree/main/third-party/

* Islet Remote Attestation: https://github.com/islet-project/remote-attestation that provides:

	* rocli: https://github.com/islet-project/remote-attestation/tree/main/tools/rocli
	  Tool for provisioning reference token and CPAK to the Veraison services.
	* realm-verifier: https://github.com/islet-project/remote-attestation/tree/main/lib/realm-verifier
      A library for veryfing RIM and REMs with reference values.
    * ratls: https://github.com/islet-project/remote-attestation/tree/main/lib/ratls
      A library implementing RaTLS protocol for attestation purposes.
    * rust-rsi: https://github.com/islet-project/remote-attestation/tree/main/lib/rust-rsi
	  A library implementing token and RSI related functionalities (fetching, parsing).

* veraison: https://github.com/veraison/services

    * Please use 5a48655b3a9c3960667ef14df7860186238b6bcd commit. Newer versions
      might not work at the moment.

# Preparation

The 3 aforementioned repositories should be checked out on the same level so it
should look like following:

    CCA/islet
	CCA/remote-attestation
	CCA/services   (this is veraison repository)

Now run `make` inside the `CCA/islet/examples/veraison` directory. This compiles
some tools that will be used for this demo and places them inside proper
directories. It also copies the `root-ca.crt` used by `realm-application`.

	CCA/islet/examples/veraison $ make

The files installed are:

* `root-ca.crt` copied to `CCA/islet/out/shared`
* `rsictl` installed in `CCA/islet/out/shared/bin`
* `realm-application` installed in `CCA/islet/out/shared/bin`
* `rocli` installed in `CCA/islet/examples/veraison/bin`
* `reliant-party` installed in `CCA/islet/examples/veraison/bin`

# Provisioning

This is emulated by generating CPAK public key using one of camellia-hes
utilities:

    CCA/islet/hes/cpak-generator $ cargo run

This will by default generate a CPAK using dummy GUK and dummy BL2 hash files
from `CCA/islet/hes/res` directory and save both key binary and PEM format
respectively as:

    CCA/islet/hes/out/cpak_public.bin
    CCA/islet/hes/out/cpak_public.pem

# Gathering measurements

There are 2 things we need to measure here. Platform and realm.

## Plaftorm measurement

The platform measurement is done by getting the whole CCA token. Platform
measurements are saved there.

This is performed by some specifically prepared realm (e.g. one provided by
`CCA/islet/scripts/fvp-cca`). To do this do the following:

    CCA/islet $ ./scripts/init.sh
    CCA/islet $ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=islet --hes --rmm-log-level info

The first command will initialize the scripts and download all required
components. The second command will build the platform and the realm and run the
FVP emulator and HES application.

If run under X environment terminals should open with telnet 5000/5003. If not
we can run those telnets manually on two separate terminals:

    $ telnet localhost 5000
    $ telnet localhost 5003

Port 5000 is the main terminal with console. 5003 is RMM. We don't need the
output of the second one, but the telnet itself is necessary for FVP to work
properly (buffering reasons).

When the FVP linux is booted we need to run the realm:

    $ ./launch-realm.sh

This will take a lot of time (FVP is slow). Wait until you have a realm
loaded. Then load RSI module and get the token:

    Welcome to Buildroot
    buildroot login: root

    # cd /shared
    shared # insmod rsi.ko
    shared # ./bin/rsictl attest -o token.bin

For the token its challenge value will be randomized, but in here it doesn't
matter. Now we can kill the FVP (ctrl-c on the FVP terminal). Eventually the
following command may be required as FVP doesn't always close cleanly:

    $ pkill -9 -i fvp

The generated token is saved as the following file:

    CCA/islet/out/shared/token.bin

## Realm measurement

Realm measurement is done by generating a json file containing realm information
that will be fed to realm verifier.

This is performed by a small helper program called `kvmtool-rim-measurer`. It basically
runs a modified lkvm tool that calculates and displays the RIM
value. The process looks as follows:

* generate/get the realm you want to use (for now generated by fvp-cca script,
  those files can be taken from `CCA/islet/out/shared`,
  `Image.realm initramfs-realm.cpio.gz realm.sh`)
* Build the kvmtool-rim-measurer tool according to the description https://github.com/islet-project/assets/blob/3rd-kvmtool-rim-measurer/BUILD-RIM-MEASURER
* Create a dedicated directory for realm files (e.g. `CCA/islet/out/rim-extractor`) and copy the realm files we want to measure to that folder
* copy the resulting `lkvm-rim-measurer` to the `CCA/islet/out/rim-extractor` folder
* substitute `lkvm` to `lkvm-rim-measurer` in the `CCA/islet/out/rim-extractor/realm.sh` script
* get into the `CCA/islet/out/rim-extractor` folder and run the `realm.sh` script
* The `lkvm-rim-measurer`` will display the resulting RIM (e.g. RIM: F58AF6D6A022F113627B1E0B1E0D9B9A1BFB460207AC29721E84BCEF4B4F5CE08351684444BC11CF329D1D4C807BB621807916C2DF4F56B7326E8D16692546A8)

Create a `realm.json` file according to the below template and replace the `TO_BE_REPLACED` term with the extracted RIM value.

```json
{
    "version": "0.1",
    "issuer": {
        "name": "Samsung",
        "url": "https://cca-realms.samsung.com/"
    },
    "realm": {
        "uuid": "f7e3e8ef-e0cc-4098-98f8-3a12436da040",
        "name": "Data Processing Service",
        "version": "1.0.0",
        "release-timestamp": "2024-11-27T05:21:31Z",
        "attestation-protocol": "HTTPS/RA-TLSv1.0",
        "port": 8088,
        "reference-values": {
            "rim": "TO_BE_REPLACED",
            "rems": [
                [
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000"
                ],
                [
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "7d43aefe4c6a955cd0753bccee2e707232d2b44b84c4607ac925597419ac104d",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "9e6f6535ee6cf18be0eae95d0a2fd6876ccdc216a172e8f15607fe1a814d0b6c"
                ]
            ],
            "hash-algo": "sha-256"
        }
    }
}
```

The resulting json should be saved as the following file:

    CCA/islet/out/rim-extractor/realm.json

Caveat: only RIM is supported for now, the REMs are placeholders.

# Provisioning/Measurement summary

Those 2 processes should end with the following things

* Prepared realm that won't be modified anymore:
  `Image.realm initramfs-realm.cpio.gz realm.sh`
  For now we use the one generated by fvp-cca
* Public CPAK key: `cpak_public.bin cpak_public.pem`
* Platform measurement: `token.bin`
* Realm measurement: `realm.json`

Those token and measurement files should be _sent_ to verification services
using a _safe_ communication channel.

# Running realm for verification purposes

This is done in almost the same way we run realm to get the token.

Run the FVP with HES and network this time. Use the --run-only param from now on
not to regenerate the realm anymore so our measurements won't get stale:

    CCA/islet $ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=islet --hes --rmm-log-level info --run-only

When FVP is booted run the realm:

    # ./launch-realm.sh net

Inside the realm you need to do the following:

* configure the network
* load the RSI module
* set the date for the certificates to work properly

This is how it looks:

    Welcome to Buildroot
    buildroot login: root

    # cd /shared
	shared # ./set-realm-ip.sh
    shared # insmod rsi.ko
	shared # date 120512002023

# Running and provisioning verification services (Veraison, realm-verifier)

First of all, before deploying Veraison, apply a patch to Veraison code (https://github.com/veraison/services):

    CCA/services $ cat ../islet/examples/veraison/veraison-patch | git apply

Then it's possible to deploy a Veraison Docker and source some useful
commands from veraison env file:

    CCA/services $ make docker-deploy
    CCA/services $ source deployments/docker/env.bash

Check if all 3 veraison services are running:

    $ veraison status
             vts: running
    provisioning: running
    verification: running

Now install go dependencies for rocli script:

    $ go install github.com/veraison/corim/cocli@latest
    $ go install github.com/veraison/ear/arc@latest
    $ go install github.com/veraison/evcli@latest

And run provisioning of token and cpak in PEM format:

    CCA/islet/examples/veraison/provisioning $ ./run.sh -t <path/to/token.bin> -c <path/to/cpak_public.pem>

This will provision a reference token and public CPAK to allow
Veraison verification.

It's possible to see current values stored in Veraison:

    $ veraison stores

And if required, they also should be cleared before they can be
provisioned again:

    $ veraison clear-stores

Run reliant-party, which is provisioned with `realm.json` and
acts as Reliant Party with communication to realm and Veraison
services (this binary takes several parameters, most should not be of
any concern apart from passing latest reference values in `realm.json`):

    CCA/islet/examples/veraison $ ./bin/reliant-party -r <path/to/realm.json>

If needed, '-b' option can be used to pass different network interface binding
(the default is 0.0.0.0:1337):

    CCA/islet/examples/veraison $ ./bin/reliant-party -r <path/to/realm.json> -b <LOCAL_IP:PORT>

Reliant-party awaits on given IP:PORT for communication from Realm and
utilizes our `ratls` Rust library and `realm-verifier` library (for `realm.json`
reference values verification) to verify client CCA token.

# Verification itself

On the realm side (the one we already run) just trigger the verification
process. This is done using `realm-application` (`CCA/islet/examples/veraison/realm-application`).
It will initialize RATLS connection to verification service by performing the necessary steps:

* receive challenge value from verification service
* request the token from RMM/TF-A/HES using the challenge
* send the received token to verification service
* establish safe connection if verification services agrees to do so

This is done with the following command on the realm:

    shared # ./bin/realm-application -r root-ca.crt -u <SERVER_IP:PORT>

That command will take a very long time as Realm on FVP is slow and it does
asymmetric cryptography (RSA key generation).

# Verification success

When verification succeeds, both `realm-application` and `realm-verifier` should not
output any errors. For both binaries you can set RUST_LOG
environmental variable to change log level (info, debug):

Realm:

    shared # RUST_LOG=info ./bin/realm-application -r root-ca.crt -u <SERVER_IP:PORT>

Reliant party:

    CCA/islet/examples/veraison $ RUST_LOG=info ./bin/reliant-party -r <path/to/realm.json> -b <LOCAL_IP:PORT>

With that log level realm client should report successful socket write
with 'GIT' message and verifying server should output that message.
