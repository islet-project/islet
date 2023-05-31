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
First of all, make sure you are in the root directory of ISLET and go throuth the following instructions.
In most cases, you would probably not have to customize network-releated arguments and feed them into `fvp-cca`. Using a default configuration would be sufficient.
```
# full command:
# ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=<PC Host IP> --fvp-ip=<FVP IP> --fvp-tap-ip=<FVP Tap Device IP> --realm-ip=<Realm IP> --route-ip=<Route IP>

$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm
  # this takes a default network configuration in which
  # --host-ip: 193.168.10.15
  # --fvp-ip: 193.168.10.5
  # --fvp-tap-ip: 193.168.20.20
  # --realm-ip: 193.168.20.10
  # --route-ip: 193.168.20.0
```

As of now, FVP being able to communicate through Host to external networks is out of scope. All communications between the three components must be done within the bounds of PC Host.

## A closer look at network configuration

This is how the aforementioned three components interact with each other:
```
// A default configuration
// Realm:     IP: 193.168.20.10 (static address),  Gateway: 193.168.20.20 (the tap device of FVP Host)
// FVP Host:  IP: 193.168.10.5 (static address),   Tap: 193.168.20.20
// PC Host:   IP: 193.168.10.15 (static address of tap device)

Realm <----------------> FVP Host  <-----------------> PC Host
      (tap network)  (ipv4_forward) (tap network)
```