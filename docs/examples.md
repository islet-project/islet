# Examples

This document describes examples you can play with through ARM CCA.
At the time of this writing, the only example supported is [simple_app](https://github.com/vmware-research/certifier-framework-for-confidential-computing/tree/main/sample_apps/simple_app) in [Certifier framework](https://github.com/vmware-research/certifier-framework-for-confidential-computing) maintained by VMWare.

## simple_app

[simple_app](https://github.com/vmware-research/certifier-framework-for-confidential-computing/tree/main/sample_apps/simple_app) is an example that demonstrates a simple message exchange through a secure channel established under control of VMWare's certifier framework. For readers who want to get more detail about it, please see [this document](https://github.com/vmware-research/certifier-framework-for-confidential-computing/blob/main/Doc/CertifierFramework.pdf).

It involves three instances: certifier service (attestation daemon), server-app, and client-app. The first two (certifier service and server-app) are suppposed to be running on x86_64 machines while the last one (client-app) runs on ARM CCA. This configuration is a great example to show how confidential computing can encompass from server-side TEEs (e.g., SGX/SEV) even to on-device TEE (ARM CCA).

Here is how to run simple_app in the above configuration.
TODO: add description about how to run the certifier service and the server-app on PC Host.

```
# 1. [in PC Host]run fvp-cca with a proper network configuration. To get what these arguments mean, see 'NETWORK.md'.
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=<PC Host IP> --ifname=<ethernet card name> --gateway=<gateway address> --fvp-ip=<FVP IP>

# 2. [in FVP Host] once fvp is launched, run a daemon process for packet forwarding.
$ cd qemu
$ ./rinetd -c rinetd.conf -f &

# 3. [in FVP Host] run a realm with a rootfs that contains prebuilt example binaries.
$ ./launch-realm-net.sh

# 4. [in Realm] run the client-app in a specific order
# TODO: full commands
$ cd /app
$ ./2_client_init.sh  # gets attested and gets an admission certificate used to establish a secure channel
```
