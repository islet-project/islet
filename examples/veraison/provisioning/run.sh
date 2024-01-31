#!/bin/bash

set -exuo pipefail
shopt -s expand_aliases

ROOT="$(git rev-parse --show-toplevel)"
DEMO="$ROOT/examples/veraison"
DOCKER_DIR="$DEMO/services/deployments/docker"
ROCLI="$DEMO/bin/rocli"
PROV="$DEMO/provisioning"
TOKEN="$PROV/token/token.bin"
CPAK="$PROV/claims/cpak_public.pem"
CPAK_TYPE="pkix-base64-key"
CONFIG="$PROV/config.yml"

source "$DOCKER_DIR/env.bash"

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

#if ! [ -f "$ROCLI" ]; then
#    cargo install --path "$ROOT" --root "$ROOT";
#fi

function loginfo () {
    echo -e "\e[0;32m$1\e[0m"
}

######  Generating Comids and Corim
loginfo "Creating Endorsements"

$ROCLI --config "$CONFIG" -o endorsements.json \
    --token "$TOKEN" endorsements \
    --cpak "$CPAK" \
    --cpak-type "$CPAK_TYPE"

loginfo "Endorsements:"
cat ./endorsements.json | jq

loginfo "Creating reference values"

$ROCLI --config "$CONFIG" -o refvals.json \
    --token "$TOKEN" refvals

loginfo "Refvals:"
cat refvals.json | jq

loginfo "Creating Corim"

$ROCLI --config "$CONFIG" -o corim.json \
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
cocli corim submit --corim-file=corim.cbor \
    --api-server="http://provisioning-service:8888/endorsement-provisioning/v1/submit" \
    '--media-type='\''application/corim-unsigned+cbor; profile=http://arm.com/cca/ssd/1'\'''

##### Verifying as relaying party

loginfo "Verifying token as relaying party"
evcli cca verify-as relying-party \
    --api-server=http://verification-service:8080/challenge-response/v1/newSession \
    --token=$TOKEN | tr -d '"' > ear.jwt
arc verify -p=pkey.jwk ear.jwt
