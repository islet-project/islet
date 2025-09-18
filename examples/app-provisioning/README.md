# App provisioning on islet instruction

Islet project implements an application provisioning mechanism that provides a generic way to install applications from a registry. It uses docker containers (OCI) as application image format – this way, Islet can benefit from the existing infrastructure & standards for creating and shipping applications.

The process of provisioning is handled by a realm daemon which is responsible for decrypting and mounting storage for applications as well as installing and running them. The storage is created by stacking 2 filesystems using OverlayFS so that the application can modify arbitrary files and directories - changes will be stored at the “Data.raw” disk, but the application image disk “Image.raw” will be untouched, containing only original binaries. Additionally, to provide confidentiality to the applications, design employs sealing key derivation mechanism similar to the one described in Open Profile for DICE specification. It is a layered approach, where each layer is responsible for deriving its own keys and preparing key material for the next layer - so that the lower layer keys depend on all upper ones. The storage encryption keys depend on both: the application identity and also on the hardware trust anchor taken from HES. Since key derivation is based on objects identity (not full data hash), this mechanism allows for easy application & firmware update, without breaking access to data stored in persistent storage.

# Demo instruction

All repositories here are to be cloned to some one common directory that is
referenced here as `$CCA`.

## Prepare host

If you're using docker (e.g. for the Veraison below) it disables FORWARD by
default and it might cause issues with the network configuration below. For
details see here:

https://docs.docker.com/engine/network/packet-filtering-firewalls/#docker-on-a-router

The fastest way (not necessarily the best as it's outside the scope of this
document) to remedy this is:

    sudo iptables -I DOCKER-USER -j ACCEPT

## Prepare repositories

    cd $CCA
    git clone https://github.com/islet-project/islet.git
    git clone https://github.com/islet-project/realm-manager.git
    git clone https://github.com/islet-project/realm-metadata-tool.git
    git clone https://github.com/islet-project/image-registry.git

## Prepare/initialize islet

This will initialize Islet dependencies required for it to work and compile two
rust tools used for this example (`rsictl` and `rocli`).

    cd $CCA/islet
    ./scripts/init.sh
    cd $CCA/islet/examples/app-provisioning
	make

## Prepare the provisioning framework

### Build realm image

You need to have `libdevmapper-dev` package installed (`sudo apt install libdevmapper-dev`).

    cd $CCA/realm-manager/realm
    make deps
    make compile-image

#### Copy resulting kernel and initramfs images to islet shared dir

    cp out/Image $CCA/islet/out/shared/nasz-realm
    cp out/initramfs.cpio.gz $CCA/islet/out/shared/initramfs.cpio.gz

### Build warden daemon

Warden daemon is responsible for providing resources such as disks and networking to realms. It manages realm lifetime by starting and stopping kvmtool. More details are available at [realm-manager/warden/warden_daemon](https://github.com/islet-project/realm-manager/tree/main/warden/warden_daemon).

    cd $CCA/realm-manager/warden
    make

#### Copy resulting binaries to islet shared dir

    mkdir $CCA/islet/out/shared/warden
    cp -v bin/* $CCA/islet/out/shared/warden/

## Provision the Application/Realm

### Obtain the attestation token

To obtain the attestation token we need to launch the realm and perform
attestation using RSI call. Warden daemon has a command for that.

#### Launch islet

    cd $CCA/islet
    ./scripts/fvp-cca --normal=linux-net --realm=linux --rmm=islet --rmm-log-level info --hes

#### Start the warden daemon (telnet 5000)

    export RUST_LOG=debug
    ./warden/warden_daemon -p 1337 -v ./lkvm -u /tmp/usocket12344 -d ./warden/dnsmasq -w /tmp/workdir -t 3200 --lkvm-runner --cca-enable --dns-records /image-registry.net/192.168.10.1 &

After seeing: `[SOME_DATE INFO warden_daemon::socket::unix_socket_server] Starting Unix Socket Server`

    ./warden/cmd_client -u /tmp/usocket12344

All the commands below are within the prompt of the client:

#### Obtain the token using warden daemon

    create-realm -n 1 -r 256 -k ./nasz-realm -i initramfs.cpio.gz -v 12344 -z 5156ae05-1da0-4e7b-a168-ec8d1869890e
    create-application -n light_app -v latest -i image-registry.net:1337 -o 32 -d 32 -r 5156ae05-1da0-4e7b-a168-ec8d1869890e
    fetch-attestation-token -r 5156ae05-1da0-4e7b-a168-ec8d1869890e -o token.bin

The token will saved as `$CCA/islet/out/shared/token.bin`

### Obtain the RIM

#### Using lkvm-rim-measurer tool

> [!CAUTION]
> This tool needs an update for v1.0-rel0, use the alternative method below for now

> [!WARNING]
> This section needs to be simplified

To obtain the RIM we can use the lkvm-rim-measurer tool.
Firstly, clone the lkvm-rim-measurer tool repository.

Build the libfdt library

    cd $CCA/islet/third-party/dtc
    make

Build the lkvm-rim-measurer tool

    cd $CCA/islet/third-party/kvmtool-rim-measurer/
    ./build-rim-measurer.sh

This should compile the lkvm-rim-measurer executable (located in $CCA/islet/third-party/kvmtool-rim-measurer/ directory).

Copy the linux and initramfs images to the rkvmtool-rim-measurer directory:

    cp $CCA/realm-manager/realm/out/Image .
    cp $CCA/realm-manager/realm/out/initramfs.cpio.gz .

Create a dummy disk:

    touch dummy-disk.img

Launch the lkvm-rim-measurer tool to obtain the RIM:

    ./lkvm-rim-measurer run \
        -c 1 \
        -k Image\
        -i initramfs.cpio.gz \
        -m 256 \
        -n tapif=tap100,guest_mac=52:55:00:d1:55:02 \
        --vsock 12344 \
        --console serial \
        --irqchip=gicv3 \
        --disable-sve \
        --debug \
        --realm \
        --measurement-algo=sha256 \
        -d dummy-disk.img \
        --islet

The tool should display the RIM at the last line e.g.:

    ...
    RIM: EB89CD86CEC19ABA5008E9380361362DBE5E4A5EBC01869166EEDD206840BD410000000000000000000000000000000000000000000000000000000000000000

For the sha256 measurement algorithm, save the first 64 hexadecimal characters of RIM for the further use.

#### Alternatively extract the RIM from the token obtained earlier

    cd $CCA/islet/examples/app-provisioning
    ./bin/rsictl verify -i $CCA/islet/out/shared/token.bin | grep "Realm initial measurement"

The RIM value will be printed between `[]` characters:

    Realm initial measurement      (#44238) = [216ea683d4ddb767c8f7be437832dc24a9692bff014eceb36ecb7a44e75d121c]

### Create provisioning files with RIM

```
cd $CCA/islet/examples/app-provisioning
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
```

### Prepare Metadata

For details see https://github.com/islet-project/realm-metadata-tool

    cd $CCA/realm-metadata-tool
    openssl ecparam -genkey -name secp384r1 -noout -out realm-vendor.pem
    cargo run -- create -m $CCA/islet/examples/app-provisioning/metadata.yaml -k realm-vendor.pem -o metadata.bin

#### Copy resulting metadata binary to islet shared dir

    cp metadata.bin $CCA/islet/out/shared

### Prepare and provision Veraison

#### Generate the CPAK key

    cd $CCA/islet/hes/cpak-generator
    cargo run

This will by default generate a CPAK using dummy GUK and dummy BL2 hash files
from `$CCA/islet/hes/res` directory and save the key as
`$CCA/islet/hes/out/cpak_public.pem`.

#### Bootstrap the Veraison service

The following command requires a working `docker` and `go` installations. It
will install the Veraison service and cmd line tools: `arc` and `rocli`.

    cd $CCA/islet/examples/app-provisioning/veraison
    ./bootstrap.sh

#### Provision the Veraison service

Make sure to copy the files into the current directory as shown below. otherwise
the Veraison docker tools won't be able to find them.

    cd $CCA/islet/examples/app-provisioning/veraison/provision
    cp $CCA/islet/hes/out/cpak_public.pem .
    cp $CCA/islet/out/shared/token.bin .
    ./run.sh -t token.bin -c cpak_public.pem

## Setup image registry and example application

### Compile example app #1

> [!CAUTION]
> TODO: create a Makefile here

> [!WARNING]
> The rest of the document uses the application from example #2, so unless doing
> something custom it's preferred to use that one.

    cd $CCA/realm-manager/realm/example-application
    docker build . -t example_app
    docker image save -o example_app.tar example_app:latest
    cd $CCA/image-registry
    mkdir registry/example_app
    cp $CCA/realm-manager/realm/example-application/example_app.tar registry/example_app
    cd registry/example_app
    tar xf example_app.tar
    rm example_app.tar

> [!NOTE]
> It is assumed that the recent versions of Docker are used, which by default produce OCI images.
> To generate OCI images on older docker version firstly you need to install docker-buildx package on your system (e.g. "apt install docker-buildx").
> Then, create a builder:
>
>   docker buildx create --use
>
> Next, run the build process:
>
>   docker buildx build --output type=oci,dest=example_app.tar . -f Dockerfile --tag=latest
>
> When signing the application using the `$CCA/image-registry/ir-sign` tool, you need to
> add write permission to files located in the blobs/sha256 folder.

### Compile example app #2

You need ubuntu's aarch64 compiler and bear (`sudo apt install bear gcc-aarch64-linux-gnu`)

    cd $CCA/image-registry/registry/light_app
    make

### Sign the applications

For detailed instructions see: https://github.com/islet-project/image-registry/tree/main/ir-sign

    cd $CCA/image-registry/ir-sign
    # openssl ecparam -name secp384r1 -genkey -noout -out app-vendor.der -outform DER   # (alternative way)
    cargo run -- gen-key -o app-vendor.der
    cargo run -- sign-image -a light_app -d latest -v app-vendor.der -x $CCA/realm-manager/realm/keys/root-ca.prv

### Run image registry

    cd $CCA/image-registry/ir-server
    cargo run -- -t ra-tls -j $CCA/islet/examples/app-provisioning/reference.json

## Launch islet

    cd $CCA/islet
    ./scripts/fvp-cca --normal=linux-net --realm=linux --rmm=islet --rmm-log-level info --hes

## Setup network in normal world linux (paste this in telnet 5000 console)

    echo 1 > /proc/sys/net/ipv4/ip_forward
    echo 'nameserver 8.8.8.8' > /etc/resolv.conf
    ping -c 3 1.1.1.1
    nslookup www.google.com

## Setting up realm (telnet 5000)

### Start warden daemon

    export RUST_LOG=debug
    ./warden/warden_daemon -p 1337 -v ./lkvm -u /tmp/usocket12344 -d ./warden/dnsmasq -w /tmp/workdir -t 3200 --lkvm-runner --cca-enable --dns-records /image-registry.net/192.168.10.1 &

After seeing: `[SOME_DATE INFO warden_daemon::socket::unix_socket_server] Starting Unix Socket Server`

    ./warden/cmd_client -u /tmp/usocket12344

All the commands below are within the prompt of the client:

### Define realm

    create-realm -n 1 -r 256 -k ./nasz-realm -i initramfs.cpio.gz -v 12344 -z 5156ae05-1da0-4e7b-a168-ec8d1869890e -d ./metadata.bin

### Define an application to provision (choose app below with -n)

    create-application -n light_app -v latest -i image-registry.net:1337 -o 32 -d 32 -r 5156ae05-1da0-4e7b-a168-ec8d1869890e

### Start realm

    start-realm -r 5156ae05-1da0-4e7b-a168-ec8d1869890e

We should observe the realm starting sucessfully, image-registry providing the
light_app application through RA-TLS and the application being run inside the
realm with:

    INFO  [app_manager::launcher::handler] Application stdout: Example Application

### Stop realm

    stop-realm -r 5156ae05-1da0-4e7b-a168-ec8d1869890e
