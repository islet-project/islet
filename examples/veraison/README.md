# Software attestation

## Why does it matter?

Software attestation is the process of certifying that some program of interest possess certain properties. Typically, it boils down to checking whether the hashes of binaries match the values provided by the developers. Moreover, in the field of confidential computing it is useful to assert the trustworthiness of the environment our program is executing in. This is essential from a couple of perspectives:

* As a software engineering company you can ensure that the virtual machines that will be running the program aren't malicious and weren't modified by a third party.
* As a user you can be sure that the secrets used to authenticate and provide encryption layer in the application are stored securely.

## How does it work?

There are two ways of performing software attestation:

* `Local attestation` where the root of trust is derived from hardware and the device is able to attest itself (this is yet to be implemented in islet).
* `Remote attestation` where the root of trust is partly provided by a third party server called the `reliant party` that implements an attestation protocol (Veraison is such software).

In detail the protocol implemented by islet is based on the `TLS` protocol which handles the handshake and the attestation is implemented by the custom certificate creation and validation procedures.
In the simplest example, which is implemented by islet, the `reliant party` implements the `TLS` server with a self-signed root certificate and a custom certificate verifier.
The client in this case is a application running inside a secure realm. It implements the `TLS` client with the custom certificate generator that creates a self-signed certificate with the attestation token embedded in the optional field.
As a reminder, an attestation token is a set of claims provided by the execution environment that will be checked by the `reliant party`.
Additionally, to prevent replay attacks the server will generate a random challenge that the client is expected to embed inside the token. When the `TLS` 3-way handshake has been finished successfully, the software is attested and the opened TCP connection can be used to transfer sensitive data or other software.


## Implementation details
![diagram](./diagram.svg)

#### Legend
* `Islet` (this project) is a Realm Management Monitor implemented in Rust it is used to manage realms and generate attestation tokens in the upcoming ARMv9 architecture.
* `RaTlsClient` is the client of the modified `TLS` protocol which uses a custom cert generator that embeds the attestation token.
* `RaTlsServer` is the server of the modified `TLS` protocol which implements custom cert verifier that uses `Veraison` and `Realm measurements` to attest the software running inside the realm.
* `Verification service` it's the actual service in the `Veraison` project responsible for attesting software, currently it only checks the platform part of the token.
* `Realm measurements` is a data store holding the trusted realm measurements, so that the `RaTlsServer` can check the realm part of the token (as mentioned `Veraison` only attests the platform part).

#### Attestation flow

* The attestation is initiated by the `RaTlsClient` creating a `TCP` connection to the `RaTlsServer` and starting the `TLS` 3-way handshake.
* The `RaTlsServer` send a challenge to the client (it's a 64bit number used to protect against replay attacks).
* The `RaTlsClient` provides `Islet` with the challenge and retrieves a signed attestation token containing:
    * platform measurements (bootloaders measurements, `Islet` measurements itself, etc...)  
    * signature signed by the `CPAK` or `Platform key`
    * realm measurements
    * realm challenge which is the challenge we got from `RaTlsServer`
    * signature signed by `RAK` or `Realm attestation key`
* The `RaTlsClient` creates a self-signed `SSL` certificate with the token embedded as a `X509` extension and provides it to the server.
* The `RaTlsServer` extracts the token from the certificate and validates the challenge.
* Next it uses `Verification service` to validate the platform part of the token.
* At last the `Realm measurements` are used to check the realm part.
* If all check finish successfully, the 3-way handshake concludes and an encrypted `TCP` connection between the `RaTlsClient` and `RaTlsServer` is opened and ready to transfer sensitive data.

For more detailed instruction refer to [RUN](./RUN.md). It contains a step by step guide of running the attestation demo using `Islet`.
