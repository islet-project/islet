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