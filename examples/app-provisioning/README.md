# App provisioning on islet instruction

Islet project implements an application provisioning mechanism that provides a generic way to install applications from a registry. It uses docker containers (OCI) as application image format – this way, Islet can benefit from the existing infrastructure & standards for creating and shipping applications.

The process of provisioning is handled by a realm daemon which is responsible for decrypting and mounting storage for applications as well as installing and running them. The storage is created by stacking 2 filesystems using OverlayFS so that the application can modify arbitrary files and directories - changes will be stored at the “Data.raw” disk, but the application image disk “Image.raw” will be untouched, containing only original binaries. Additionally, to provide confidentiality to the applications, design employs sealing key derivation mechanism similar to the one described in Open Profile for DICE specification. It is a layered approach, where each layer is responsible for deriving its own keys and preparing key material for the next layer - so that the lower layer keys depend on all upper ones. The storage encryption keys depend on both: the application identity and also on the hardware trust anchor taken from HES. Since key derivation is based on objects identity (not full data hash), this mechanism allows for easy application & firmware update, without breaking access to data stored in persistent storage.

# Demo instruction

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

#### Copy resulting kernel and initramfs images to islet shared dir

    cp out/Image $ROOT/islet/out/shared/nasz-realm
    cp out/initramfs.cpio.gz $ROOT/islet/out/shared/initramfs.cpio.gz

### Build warden daemon

Warden daemon is responsible for providing resources such as disks and networking to realms. It manages realm lifetime by starting and stopping kvmtool. More details are available at [realm-manager/warden/warden_daemon](https://github.com/islet-project/realm-manager/tree/main/warden/warden_daemon).

    cd $ROOT/realm-manager/warden
    make

#### Copy resulting binaries to islet shared dir

    mkdir $ROOT/islet/out/shared/warden
    cp -v bin/* $ROOT/islet/out/shared/warden

## Provision the Application/Realm

### Obtain the RIM

To obtain the RIM we can use the lkvm-rim-measurer tool.
Firstly, clone the lkvm-rim-measurer tool repository.

Build the libfdt library

    cd $ROOT/islet/third-party/dtc
    make

Build the lkvm-rim-measurer tool

    cd $ROOT/islet/third-party/kvmtool-rim-measurer/
    ./build-rim-measurer.sh

This should compile the lkvm-rim-measurer executable (located in $ROOT/islet/third-party/kvmtool-rim-measurer/ directory).

Copy the linux and initramfs images to the rkvmtool-rim-measurer directory:

	cp $ROOT/realm-manager/realm/out/Image .
	cp $ROOT/realm-manager/realm/out/initramfs.cpio.gz .

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

> [!NOTE]
> It is assumed that the recent versions of Docker are used, which by default produce OCI images.
> To generate OCI images on older docker version firstly you need to install docker-buildx package on your system (e.g. "apt install docker-buildx").
> Then, create a builder:
>
>	docker buildx create --use
>
> Next, run the build process:
>
>	docker buildx build --output type=oci,dest=example_app.tar . -f Dockerfile --tag=latest
>
> When signing the application using the `$ROOT/image-registry/ir-sign` tool, you need to
> add write permission to files located in the blobs/sha256 folder.

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

## Setup network in normal world linux (paste this in telnet 5000 console)

Replace `106.10.9.180` with some DNS server that works for you.

    echo 1 > /proc/sys/net/ipv4/ip_forward
    echo 'nameserver 106.10.9.180' > /etc/resolv.conf
    ping -c 3 1.1.1.1
    nslookup www.google.com

## Setting up realm (telnet 5000)

### Start warden daemon

    export RUST_LOG=debug
    ./warden/warden_daemon -p 1337 -v ./lkvm -u /tmp/usocket12344 -d ./warden/dnsmasq -w /tmp/workdir -t 3200 --lkvm-runner --cca-enable --dns-records /image-registry.net/192.168.10.1 &

After seeing: `[SOME_DATE INFO warden_daemon::socket::unix_socket_server] Starting Unix Socket Server`

    ./warden/cmd_client -u /tmp/usocket12344

All the commands below are within the UI of the client:

### Define realm

    create-realm -n 1 -r 256 -k ./nasz-realm -i initramfs.cpio.gz -v 12344 -z 5156ae05-1da0-4e7b-a168-ec8d1869890e -d ./metadata.bin

### Define an application to provision (choose app below for -n)

    create-application -n light_app -v latest -i image-registry.net:1337 -o 32 -d 32 -r 5156ae05-1da0-4e7b-a168-ec8d1869890e

### Start realm

    start-realm -r 5156ae05-1da0-4e7b-a168-ec8d1869890e

### Stop realm

    stop-realm -r 5156ae05-1da0-4e7b-a168-ec8d1869890e
