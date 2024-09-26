# Network configuration

## Enable the capability of networking

In the environment of FVP-based emulation, there are many components involved so enabling a network is not an easy task.
The three components involved are:
- (1) *PC Host* (Ubuntu only supported at this time of writing), which tries to launch FVP Host.
- (2) *FVP Host*, which is going to be running as a guest machine of PC Host.
- (3) *Realm*, which is going to be launched by FVP Host and acts as a guest to FVP Host.

In our network configuration, each of the three has different static IP address so that they can communicate with each other by specifying a proper destination IP address.
Under this setting, any of the three can act as either server or client.

And here is how to make "*FVP Host* and *Realm*" capable of communicating through to *PC Host*.
First, make sure you are in the root directory of Islet and go through the following instructions.
In most cases, it would be sufficient to use a default configuration but `--host-ip` and `--ifname`.
```
# full command:
# ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=islet --hes --no-telnet --rmm-log-level=info --ifname=<the network interface name in your PC host> --host-ip=<the IP of your PC host>

$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=islet --hes --no-telnet --rmm-log-level=info --ifname=enp3s0 --host-ip=111.222.333.15
  # this takes a default network configuration in which
  # --host-ip: put in the IP address of your PC Host (111.222.333.15)
  # --ifname: put in the network interface name of your PC Host (enp3s0)
  # --host-tap-ip: 193.168.10.1 (default value)
  # --fvp-ip: 193.168.10.5 (default value)
  # --fvp-tap-ip: 193.168.20.1 (default value)
  # --realm-ip: 193.168.20.10 (default value)
  # --route-ip: 193.168.20.0 (default value)
```
In this setting, both FVP Host and Realm are able to connect to external networks (i.e., internet) through PC Host's network interface you specify through `--ifname`.

## A closer look at network configuration

This is how the aforementioned three components interact with each other:
```
// A default configuration
// Realm:     IP: 193.168.20.10 (static address),  Gateway: 193.168.20.1 (the tap device of FVP Host)
// FVP Host:  IP: 193.168.10.5 (static address),   Gateway: 193.168.10.1 (the tap device of PC Host)
// PC Host:   IP: 111.222.333.15 (a real IP address + tap device + Source NAT)

Realm <----------------> FVP Host  <-----------------> PC Host
       (tap network)    (ipv4_forward)   (tap network)
```

