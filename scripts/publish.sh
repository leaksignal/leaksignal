#!/bin/bash
set -e
here=$(realpath $(dirname "$0"))
cd "$here/../leakpolicy"

cargo publish

cd "$here/../leaksignal"

cargo publish
