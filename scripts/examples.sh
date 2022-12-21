#!/bin/bash

set -e; # Fail fast
# shellcheck shell=bash

# examples that we can't run because they require some additional infra, docker or ledger for example
ignored=(
  "examples-contracts:contract_deploy_moonbeam"
  "examples-providers:ipc"
  "examples-wallets:ledger"
  "examples-wallets:trezor"
  "examples-wallets:yubi"
  "examples-transactions:txn_remove_liquidity"
)

example_crates=$(cargo metadata --format-version 1 | 
                 jq -c '.workspace_members' | 
                 jq -r 'map(select(startswith("examples")) |
                 sub("\\s.*$";"")) | .[]')

for crate in $example_crates; do 
    # Remove "examples-" prefix from crate name (e.g. examples-contracts => contracts)
    cratedir="${crate#examples-}"
    srcdir="examples/$cratedir/examples"
    # Retrieve all example files in crate:
    # Transform the absolute path into the filename (e.g. some-path/deploy_anvil.rs => deploy_anvil)
    example_files=$(find $srcdir -type f -name '*.rs' -exec basename {} \; | sed 's/\.[^.]*$//')

    for file in $example_files; do
        # Build the example
        echo "Building example: $crate:$file"
        cargo build -p $crate --example $file

        # Run example
        if [[ "${ignored[*]}" =~ $(basename "$crate:$file") ]]; then
            echo "skipping: $crate:$file"
            continue
        fi
        echo "running $crate:$file"
        cargo run -p $crate --example $file
    done
done
