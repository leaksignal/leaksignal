#!/bin/bash
set -e
here=$(realpath $(dirname "$0"))
cd "$here/../.."

cargo build --release

cd "$here"

POLICY_FILE=${POLICY_FILE:-./config/local_policy.yaml}

export POLICY=$(cat $POLICY_FILE | sed -e 's/^/            /')

export ENVOY_YAML=.envoy_local.gen.yaml
ENVOY_CONFIG=${ENVOY_CONFIG:-./config/envoy_local.yaml}
envsubst < $ENVOY_CONFIG > ./config/$ENVOY_YAML

cp ../../target/wasm32-unknown-unknown/release/leaksignal.wasm .
docker-compose build --no-cache
rm -f leaksignal.wasm
docker-compose up
