#!/bin/bash
set -e
here=$(realpath $(dirname "$0"))
cd "$here/../.."

if [ -z ${API_KEY+x} ] ; then
    echo "API_KEY env var required"
    exit 1
fi

if [ -z ${DEPLOYMENT_NAME+x} ] ; then
    echo "DEPLOYMENT_NAME env var required"
    exit 1
fi

cargo build --release

cd "$here"

export ENVOY_YAML=.envoy.gen.yaml
ENVOY_CONFIG=${ENVOY_CONFIG:-./config/envoy.yaml}
envsubst < $ENVOY_CONFIG > ./config/$ENVOY_YAML

cp ../../target/wasm32-unknown-unknown/release/leaksignal.wasm .
docker-compose build --no-cache
rm -f leaksignal.wasm
docker-compose up
