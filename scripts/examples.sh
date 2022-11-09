set -e
# shellcheck shell=bash

# examples that we can't run because they require some additional infra, docker or ledger for example
ignored=(
  "moonbeam_with_abi"
  "ipc"
  "ledger"
  "paginated_logs"
  "subscribe_logs"
  "trezor"
  "yubi"
  "remove_liquidity"
)

# run all examples
for file in examples/*.rs; do
  name="$(echo "$file" | cut -f 1 -d '.')"
  if [[ "${ignored[*]}" =~ $(basename "$name") ]]; then
    echo "skipping: $file"
    continue
  fi
  echo "running: $file"
  cargo r -p ethers --example "$(basename "$name")" --features "ethers-solc rustls ws"
done
