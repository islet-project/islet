# App provisioning on islet instruction

All repositories here are to be cloned to some one common directory that is
referenced here as `$ROOT`.

## Prepare islet (the branch is important)

    cd $ROOT
    git clone -b app-provisioning https://github.com/islet-project/islet.git

### Initialize islet

    cd $ROOT/islet
    ./scripts/init.sh

## Prepare the provisioning framework

    cd $ROOT
    git clone https://github.com/islet-project/realm-manager.git

### Build realm image

    cd $ROOT/realm-manager/realm
    make deps
    make compile-image

#### Copy resulting kernel image to islet shared dir

    cp linux/arch/arm64/boot/Image $ROOT/islet/out/shared/nasz-realm

### Build warden daemon

    cd $ROOT/realm-manager/warden
    make

#### Copy resulting binaries to islet shared dir

    mkdir $ROOT/islet/out/shared/warden
    cp -v bin/* $ROOT/islet/out/shared/warden

## Provision the Application/Realm

### Obtain the RIM (TODO)

Can be read when launching the realm in the Islet logs (telnet 5003)

    [INFO]islet_rmm::rmi::realm -- RIM: 695924ba77cea5e06c4597b5a9db058ceeb97fc8a0a1fd9727248da02fb3958d
    [INFO]islet_rmm::rmi::realm -- RIM_HASH_ALGO: sha256

### Create provisioning files with RIM

    cd $ROOT
    export RIM="PASTE_THE_OBTAINED_RIM_HEX_STRING_HERE"

    cat > metadata.yaml << EOF
    realm_id: "com.company.realm"
    version: "1.0.0"
    svn: 1
    rim: "$RIM"
    hash_algo: SHA256
    EOF

    cat > reference.json << EOF
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
            "release-timestamp": "2024-09-09T05:21:31Z",
            "attestation-protocol": "HTTPS/RA-TLSv1.0",
            "port": 8088,
            "reference-values": {
                "rim": "$RIM",
                "rems": [
                    [
                        "0000000000000000000000000000000000000000000000000000000000000000",
                        "0000000000000000000000000000000000000000000000000000000000000000",
                        "0000000000000000000000000000000000000000000000000000000000000000",
                        "0000000000000000000000000000000000000000000000000000000000000000"
                    ]
                ],
                "hash-algo": "sha-256"
            }
        }
    }
    EOF

### Prepare Metadata

For details see https://github.com/islet-project/realm-metadata-tool

    cd $ROOT
    git clone https://github.com/islet-project/realm-metadata-tool
    cd $ROOT/realm-metadata-tool
    openssl ecparam -genkey -name secp384r1 -noout -out private.pem
    cargo run -- create -m $ROOT/metadata.yaml -k private.pem -o metadata.bin

#### Copy resulting metadata binary to islet shared dir

    cp metadata.bin $ROOT/islet/out/shared

## Setup image registry and example application

### Setup image registry

    cd $ROOT
    git clone https://github.com/islet-project/image-registry.git

### Compile example app #1 (TODO: create a Makefile here)

    cd $ROOT/realm-manager/realm/example-application
    docker build . -t example_app
    docker image save -o example_app.tar example_app:latest
    cd $ROOT/image-registry
    mkdir registry/example_app
    cp $ROOT/realm-manager/realm/example-application/example_app.tar registry/example_app
    cd registry/example_app
    tar xf example_app.tar
    rm example_app.tar

### Compile example app #2

You need ubuntu's aarch64 compiler and bear (`sudo apt install bear gcc-aarch64-linux-gnu`)

    cd $ROOT/image-registry/registry/light_app
    make

### Sign the applications

For detailed instructions see: https://github.com/islet-project/image-registry/tree/main/ir-sign

    cd $ROOT/image-registry/ir-sign
    cargo run -- gen-key -o vendor.prv
    cargo run -- sign-image -a light_app -d latest -v vendor.prv -x $ROOT/realm-manager/realm/keys/root-ca.prv

### Run image registry

    cd $ROOT/image-registry/ir-server
    cargo run -- -t ra-tls -j $ROOT/reference.json

## Launch islet

*WARNING*: Make sure you use the branch mentioned above

    cd $ROOT/islet
    ./scripts/fvp-cca --normal=linux-net --realm=linux --rmm=islet --rmm-log-level info --hes

## After launching islet (on the host, enough to do once per reboot)

    sudo iptables -A FORWARD --src 192.168.10.0/24 -j ACCEPT
    sudo iptables -A FORWARD --dst 192.168.10.0/24 -j ACCEPT
    sudo iptables -t nat -A POSTROUTING -j MASQUERADE -s 192.168.10.0/24
    sudo ip addr del 193.168.10.15/24 dev armbr0
    sudo ip addr add 192.168.10.15/24 dev armbr0

## Setup network in normal world linux (paste this in telnet 5000 console)

Replace `106.10.9.180` with some DNS server that works for you.

    ip link set eth0 up
    ip addr add 192.168.10.40/24 dev eth0
    ip route add default via 192.168.10.15
    echo 1 > /proc/sys/net/ipv4/ip_forward
    echo 'nameserver 106.10.9.180' > /etc/resolv.conf
    ping -c 3 1.1.1.1
    nslookup www.google.com

## Setting up realm (telnet 5000)

### Start warden daemon

    export RUST_LOG=debug
    ./warden/warden_daemon -p 1337 -v ./lkvm -u /tmp/usocket12344 -d ./warden/dnsmasq -w /tmp/workdir -t 3200 --lkvm-runner --cca-enable --dns-records /image-registry.net/192.168.10.15 &

After seeing: `[SOME_DATE INFO warden_daemon::socket::unix_socket_server] Starting Unix Socket Server`

    ./warden/cmd_client -u /tmp/usocket12344

All the commands below are within the UI of the client:

### Define realm

    create-realm -n 1 -r 256 -k ./nasz-realm -v 12344 -z 5156ae05-1da0-4e7b-a168-ec8d1869890e -d ./metadata.bin

### Define an application to provision (choose app below for -n)

    create-application -n light_app -v latest -i image-registry.net:1337 -o 32 -d 32 -r 5156ae05-1da0-4e7b-a168-ec8d1869890e

### Start realm

    start-realm -r 5156ae05-1da0-4e7b-a168-ec8d1869890e

### Stop realm

    stop-realm -r 5156ae05-1da0-4e7b-a168-ec8d1869890e
