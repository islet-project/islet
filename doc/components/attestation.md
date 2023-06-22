# Attestation

Remote Attestation (RA) is the key of confidential computing platform, which is basically a method that convinces to *verifier* that a program (*attester*) is running on a proper confidential computing platform (e.g., SGX or ARM CCA).
Unfortunately, at the beginning of ARM TrustZone which has been widely adopted by mobile device vendors up to this date, it lacks the support of RA in the form of a specification. Some research papers (e.g., [SecTEE](https://dl.acm.org/doi/10.1145/3319535.3363205)) have proposed a method to bring RA into TrustZone. However, due to the lack of standardization, RA comes in vendor-specific forms.

To address this problem, ARM CCA has been designed from the beginning, having RA in mind, and comes with a  document ([ARM CCA Security Model](https://developer.arm.com/documentation/DEN0096/latest)). On top of it, the attestation token format and the architecture described in that document align well with *[Remote ATtestation procedureS (RATS)](https://datatracker.ietf.org/wg/rats/about/)* specification, which is in active development to standardize RA stuff. This is a good thing as it implies that RA of CCA is not tightly coupled with a specific protocol, rather can connect to any attestation protocol.

## Report

An attestation report (shortly, *report*) is an *evidence* produced by *attester* and consumed by *verifier*. In ARM CCA, report consists of two different tokens:

- *CCA Platform token*: it is used to assure that *attester* is running on a secure CCA platform. It covers the measurements of CCA platform components (e.g., RMM and EL3M) and whether it is in debug state.
- *Realm token*: this token is used to hold the measurement of Realm, which is equivalent to a virtual machine that may contain kernel and root file system.

You can quickly test and see what this report looks like through [our CLI tool](https://samsung.github.io/islet/components/cli.html).

## Appraisal policy

According to RATS, there is a term named *Appraisal Policy (shortly, Policy)*, which is central to how to build a real-world service on top of RA. You can basically think of Policy as a set of rules that *verifier* wants to enforce.
For example, what tokens in a report say is basically sort of measurements signed by secure cryptographic keys. So, to build a meaningful security service around it, you have to write down Policy like "(1) Realm must have the measurement of X, (2) the measurement of CCA Platform software must be Y".

As you may notice, managing Policy is out of scope of CCA as this is inherently not dependent on CCA. Instead, there are several open-source projects that take on this capability, for example, [Veraison](https://github.com/veraison/) and [Certifier Framework](https://github.com/vmware-research/certifier-framework-for-confidential-computing/). They all aim to implement a standardized way to express and enforce Policy. (note that they sometimes are treated as a unified verification service as they are able to work across various TEEs)

ISLET might be able to work with any verification service in theory, but currently bases the ability to handle Policy on Certifier Framework.
You can see further details [here](TODO: link to certifier) on what Certifier Framework is and how ISLET uses it.
