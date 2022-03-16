# shellcheck shell=bash
# run all examples
for file in examples/*.rs; do
  name="$(echo "$file" | cut -f 1 -d '.')"
  echo "running $name"
  cargo r -p ethers --example "$(basename "$name")"
done
