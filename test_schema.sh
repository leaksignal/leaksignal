set -e
here=$(realpath $(dirname "$0"))
cd "$here"

for file in examples/policies/*; do
    echo "Validating $file"
    ajv validate -s policy.schema.json -d "$file" --spec=draft2020 || exit 1
done