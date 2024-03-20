set -e
here=$(realpath $(dirname "$0"))
cd "$here"

for file in examples/policies/*; do
    echo "Validating $file"
    yaml2json "$file" > "$file.json"
    ajv validate -s policy.schema.json -d "$file.json" --spec=draft2020 || exit 1
    rm "$file.json"
done