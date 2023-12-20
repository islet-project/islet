# Examples

This contains examples that you can play around with ISLET and show various use-cases where ISLET can play a crucial role in offering security and privacy.

## Certifier framework

The aim of ISLET is to get the most out of ARM CCA in order to protect ARM-based mobile devices,
which is orthogonal to a problem of bringing ISLET's system security capability into realistic yet complex use-cases.

In order to support such a complex real-world use-case, we rely on *[Certifier framework](https://github.com/vmware-research/certifier-framework-for-confidential-computing)* (maintained by VMWare) for a number of capabilities that include:
- specifying a proper policy that depends on what an end-user service is
- establishing secure channels between devices
- cross-platform confidential computing architecture

As *Certifier framework* is pursuing a universal confidential computing framework that is able to run across different TEEs, it offers a great level of abstraction for various TEEs including ARM CCA, which opens up an opportunity for ISLET to interact with other instances (running on other architectures like SGX or AMD SEV) in a unified fashion.

## Example-1: Confidential Machine Learning

The first example in which we see ISLET plays a crucial role is machine learning. ISLET can enable confidential machine learning where a model provider (e.g., AI company) and a data provider (the owner of a mobile device) are mutually distrusting, in collaboration with *Certifier framework*.

See [this document](./confidential-ml/README.md) to get to know better what confidential machine learning means and how to try out this example.

## Example-2: Cross-platform End-To-End Encryption (E2EE)

The second example we will discuss is
cross-platform end-to-end encryption (E2EE).
This example is based on the same scenario
as [the previous one](#example-1-confidential-machine-learning)
but focuses specifically on demonstrating how cross-platform E2EE works.
Unlike traditional E2EE,
attestation happens before the secure channel is established.
By doing this,
we ensure that both parties are trustworthy
before any sensitive data is exchanged.

In order to understand how cross-platform E2EE works, let's break down the process step by step:

1. Attestation: Before establishing a secure channel, both parties perform remote attestation to verify their integrity and authenticity. During this phase, a trusted third party (TTP) generates a unique identifier for each party, called a quote, based on their hardware configuration and software state. These quotes are signed by the TTP and can be used later during the secure channel establishment process to confirm the identity of each party.
2. Secure Channel Establishment: Once both parties have passed the attestation phase, they proceed to establish a secure channel using OpenSSL. However, unlike traditional E2EE, the secure channel is not established until after attestation has occurred. This means that both parties know they are communicating with a trusted entity before any sensitive data is transmitted over the channel.
3. Data Exchange: With the secure channel now established, both parties can begin exchanging sensitive data confidentially. Any data transmitted over the channel is encrypted using the previously agreed upon key material generated during the secure channel establishment phase. Because both parties have already proven their trustworthiness through attestation, there is no risk of unauthorized access or tampering with the data while it is in transit.

By incorporating attestation into the E2EE process,
our cross-platform E2EE adds an additional layer of security that
helps protect against potential threats posed by malicious actors or compromised systems.
This makes it ideal for use cases
where confidentiality and trust are paramount concerns,
such as financial transactions, healthcare records management, or government communications.

See [this document](./cross-platform-e2ee/README.md) to get to know better how to try out this example.
