set -e
here=$(realpath $(dirname "$0"))
cd "$here/../../leaksignal"

if [ -z ${API_KEY+x} ] ; then
    echo "API_KEY env var required"
    exit 1
fi

cargo build --release

COMMIT=$(git rev-parse --verify --short HEAD)
DATE=$(date -u '+%Y_%m_%d_%H_%M_%S')

PREFIX="${DATE}_${COMMIT}"

WASM_FILE="../target/wasm32-unknown-unknown/release/leaksignal.wasm"

export PROXY_HASH=$(sha256sum -b ${WASM_FILE} | cut -d" " -f1)
HASH_FILE=./leaksignal.sha256

echo $PROXY_HASH > $HASH_FILE

aws s3 cp $WASM_FILE s3://leakproxy/${PREFIX}/leaksignal.wasm
aws s3 cp $HASH_FILE s3://leakproxy/${PREFIX}/leaksignal.sha256

rm -f $HASH_FILE

export PROXY_URL="https://ingestion.app.staging.leaksignal.com/s3/leakproxy/${PREFIX}/leaksignal.wasm"

kubectl config use-context arn:aws:eks:us-west-2:829300478952:cluster/leaksignal-demo
envsubst < "$here/staging.yaml" | kubectl apply -f -
kubectl delete pods --all

kubectl get pods

FRONTEND_POD=$(kubectl get pods | grep frontend | sed 's/ .*//g')
echo "frontend logs: kubectl logs $FRONTEND_POD -c istio-proxy"
echo "waiting 3 seconds..."
sleep 3
kubectl logs $FRONTEND_POD -c istio-proxy -f

echo -e "\n\nfrontend logs: kubectl logs $FRONTEND_POD -c istio-proxy"
