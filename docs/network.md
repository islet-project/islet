# Network configuration

## Enable the capability of networking

In the environment of FVP-based emulation, there are many components involved so enabling a network is not an easy task.
The three components involved are:
- (1) *PC Host* (Ubuntu only supported at this time of writing), which tries to launch FVP Host.
- (2) *FVP Host*, which is going to be running as a guest machine of PC Host.
- (3) *Realm*, which is going to be launched by FVP Host and acts as a guest to FVP Host.

And here is how to make "*FVP Host* and *Realm*" capable of communicating through to *PC Host*.
First of all, make sure you are in the root directory of ISLET and go throuth the following instructions.
```
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=<PC Host IP> --ifname=<ethernet card name> --gateway=<gateway address> --fvp-ip=<FVP IP>
# e.g., ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=192.168.10.15 --ifname=eth0 --gateway=192.168.0.1 --fvp-ip=192.168.10.5
```

In the above command, you have to feed four arguments of network configuration: `--host-ip` (the IP of PC Host), `--ifname` (the name of interface), `--gateway` (the gateway address of PC Host), `--fvp-ip` (IP address that you want to assign to FVP Host).

Note that both `--host-ip` and `--fvp-ip` should be within the same network as we make use of "tap network" to enable network of FVP Host. That's because ARM FVP doesn't support outward connection (i.e., from guest to host) in user mode networking.

Also, we have not confirmed yet if it works fine with wifi adapters and it would likely be not impossible but may require slightly different configurations. (only ethernet interface has been confirmed)

## A closer look at network configuration

This is how the aforementioned three components interact with each other:
```
// An example configuration
// Realm:     IP: 192.168.33.7 (obtained by dhcp), Gateway: 192.168.33.1
// FVP Host:  IP: 110.110.11.5 (static address),   Gateway: 110.110.11.3
// PC Host:   IP: 110.110.11.3 (static address),   Gateway: 110.110.11.1

Realm -------------> FVP Host  --------------> PC Host
      (user mode)    (rinetd)   (tap network)
```

Let's walk thorugh a concret example of sending packet in order to understand how they are put together.
The example would be "*An application (App) in Realm is trying to send a packet to PC Host. What should the application do?*"

1. [*Realm*] App sends a packet to `192.168.33.1:8123`, which is the gateway of Realm.
    - The first thing App has to do is to send a packet to FVP Host and expect FVP to pass it on to PC Host. This can be done by sending a packet to the gateway of Realm.
    - The gateway address of Realm (192.68.33.1) depends on its virtual machine monitor. (192.168.33.1 is the default gateway of kvmtool)
2. [*FVP Host*] Pass the packet on to the PC Host, technically to the `110.110.11.3:8123`
    - Since the destination App wants to reach is PC Host, not FVP Host, FVP Host has to simply forward it through to PC Host.
    - We use `rinetd` to do this task. `rinetd` is a simple daemon process that receives packets from Realm and sends them to PC Host.
    - It's worth noting that the gateway of FVP Host is equal to the IP address of PC Host, which means that packets coming to the gateway will be forwarded to PC Host.
3. [*PC Host*] A server listening to the port-8123 can receive a packet that comes all the way from Realm-!
