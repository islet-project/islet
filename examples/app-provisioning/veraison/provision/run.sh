#!/bin/bash

set -exuo pipefail
shopt -s expand_aliases

ROOT="$(git rev-parse --show-toplevel)"
VERAISON="$ROOT/examples/app-provisioning/veraison"
DOCKER_DIR="$VERAISON/services/deployments/docker"
ROCLI="$VERAISON/bin/rocli"

TOKEN="token.bin"
CPAK="cpak_public.pem"
CPAK_TYPE="pkix-base64-key"

source "$DOCKER_DIR/env.bash"
export PATH="$HOME/go/bin:$PATH"

while getopts "ht:c:e:" arg; do
  case $arg in
    h)
      echo -e "Usage: ./run.sh -t <token path> -c <cpak public pem>"
      exit 0
      ;;
    t)
      TOKEN=$OPTARG
      echo "Using $TOKEN"
      ;;
    c)
      CPAK=$OPTARG
      echo "Using $CPAK"
      ;;
    e)
      CPAK_TYPE=$OPTARG
      echo "Using $CPAK_TYPE"
      ;;
  esac
done

if [ ! -r "$TOKEN" ]; then
	echo "You need a valid token file (either token.bin or pass with -t)"
	exit 1
fi

if [ ! -r "$CPAK" ]; then
	echo "You need a valid CPAK file (either cpak_public.pem or pass with -c)"
	exit 1
fi

function loginfo () {
    echo -e "\e[0;32m$1\e[0m"
}

###### Remove existing policy
loginfo "Clearing existing policy"
veraison clear-stores

###### Import policy
loginfo "Importing policy"
pocli create ARM_CCA accept-all.rego -i

######  Generating Comids and Corim
loginfo "Creating Endorsements"

"$ROCLI" --config config.yml -o endorsements.json \
    --token "$TOKEN" endorsements \
    --cpak "$CPAK" \
    --cpak-type "$CPAK_TYPE"

loginfo "Endorsements:"
cat endorsements.json | jq

loginfo "Creating reference values"

"$ROCLI" --config config.yml -o refvals.json \
    --token "$TOKEN" refvals

loginfo "Refvals:"
cat refvals.json | jq

loginfo "Creating Corim"

"$ROCLI" --config config.yml -o corim.json \
    --token "$TOKEN" corim

loginfo "Corim:"
cat corim.json | jq

###### Encoding above using cocli

loginfo "Encoding Comids into CBOR using cocli"
cocli comid create --template=endorsements.json --template=refvals.json

loginfo "Generating Corim"
cocli corim create --template=corim.json --comid=endorsements.cbor --comid=refvals.cbor

##### Provision Corim to Verasion provisioning service

loginfo "Provisioning generated Corim"
cocli corim submit --corim-file=corim.cbor -i \
    --api-server="https://provisioning-service:8888/endorsement-provisioning/v1/submit" \
    '--media-type=''application/corim-unsigned+cbor; profile=\"http://arm.com/cca/ssd/1\"'

##### Verifying as relaying party

loginfo "Verifying token as relaying party"
evcli cca verify-as relying-party \
    --api-server=https://verification-service:8080/challenge-response/v1/newSession \
    --token "$TOKEN" | tail -n 1 | tr -d '"' > ear.jwt
arc verify -p=pkey.jwk ear.jwt
