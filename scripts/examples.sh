set -e
# shellcheck shell=bash
# run all examples
for file in examples/*.rs; do
  name="$(echo "$file" | cut -f 1 -d '.')"
  cargo r -p ethers --example "$(basename "$name")"
done
