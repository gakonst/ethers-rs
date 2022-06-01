set -e
# shellcheck shell=bash

# examples that we can't run because they require some additional infra, docker for example
ignored=["examples/moonbeam_with_abi.rs"]

# run all examples
for file in examples/*.rs; do
  if [[ "${ignored[*]}" =~ $file ]]; then
      echo "skipping: $file"
      continue
  fi
  echo "running: $file"
  name="$(echo "$file" | cut -f 1 -d '.')"
  cargo r -p ethers --example "$(basename "$name")" --features = "ipc,ledger,trezor,yubi"
done
