# Certifier framework

[Certifier framework](https://github.com/vmware-research/certifier-framework-for-confidential-computing) is a framework designed to bring the ability to handle Policy into reality in an TEE-agnostic way. It offers a set of client APIs that can plug into various TEEs including ISLET (ARM CCA) and server called Ceritifer Service, which takes the role of verifying attestation reports that come from various TEEs.
ISLET currently adopts this framework to realize end-to-end confidential services that likely involve more than two different TEEs.

## What we can do with Certifier Framework

To get what we can do with Certifier Framework, we want to show you a simple example in which client and server want to communicate through a secure channel authenticated by confidential computing platforms. There are three components involved-- *certifier service* which this framework offers by default, *client* which runs on CCA, and *server* which runs on SGX or AMD SEV.
The goal of the certifier framework is to allow only pre-defined applications to pass authentication, and thus block malicious applications in the first place.

The first thing we need to do to build this service is to write down Policy, that is to say, embedding a set of claims into a format called Policy. The Policy would look like this in verbal form:
- The measurement of application trying to join this service must be one of Client_Measurement and Server_Measurement.
- Applications trying to join this service run on a secure confidential computing platform.

After making an agreement on the policy, authentic client and server are going to be launched and generate an attestation report and send it to the certifier service.
And then, the certifier service verifies that report based on the policy, in other words, verifying if that report doesn't violate any claims in the policy.
Only if properly verified, the certifier service issues *admission cert* which is going to be used to build a secure channel between client and server.
From that point on, they can trust each other and send messages to each other securely.

## A more complex example

For a more realistic case, we've built Confidential ML on top of the certifier framework.
See [this page](TODO: Link) to know what it is in more depth.
