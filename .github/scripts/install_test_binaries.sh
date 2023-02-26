#!/usr/bin/env bash
# Installs Solc and Geth binaries
# Note: intended for use only with CI (x86_64 Ubuntu, MacOS or Windows)
set -e

GETH_BUILD=${GETH_BUILD:-"1.11.2-73b01f40"}

BIN_DIR=${BIN_DIR:-"$HOME/bin"}

PLATFORM="$(uname -s | awk '{print tolower($0)}')"
if [ "$PLATFORM" != "linux" ] && [ "$PLATFORM" != "darwin" ]; then
    EXT=".exe"
fi

main() {
    mkdir -p "$BIN_DIR"
    cd "$BIN_DIR"
    export PATH="$BIN_DIR:$PATH"
    if [ "$GITHUB_PATH" ]; then
        echo "$BIN_DIR" >> "$GITHUB_PATH"
    fi

    install_geth &
    g=$!
    install_solc &
    wait $g $!

    echo ""
    echo "Installed Geth:"
    geth version
    echo ""
    echo "Installed Solc:"
    solc --version
}

# Installs geth from https://geth.ethereum.org/downloads
install_geth() {
    case "$PLATFORM" in
        linux|darwin)
            name="geth-$PLATFORM-amd64-$GETH_BUILD"
            curl -s "https://gethstore.blob.core.windows.net/builds/$name.tar.gz" | tar -xzf -
            mv -f "$name/geth" ./
            rm -rf "$name"
            chmod +x geth
            ;;
        *)
            name="geth-windows-amd64-$GETH_BUILD"
            zip="$name.zip"
            curl -so "$zip" "https://gethstore.blob.core.windows.net/builds/$zip"
            unzip "$zip"
            mv -f "$name/geth.exe" ./
            rm -rf "$name" "$zip"
            ;;
    esac
}

# Installs solc from https://binaries.soliditylang.org (https://github.com/ethereum/solc-bin)
install_solc() {
    bins_url="https://binaries.soliditylang.org"
    case "$PLATFORM" in
        linux)  bins_url+="/linux-amd64";;
        darwin) bins_url+="/macosx-amd64";;
        *)      bins_url+="/windows-amd64";;
    esac

    list=$(curl -s "$bins_url/list.json")
    # use latest version
    if [ -z "$SOLC_VERSION" ]; then
        SOLC_VERSION="$(echo "$list" | jq -r ".latestRelease")"
    fi
    bin=$(echo "$list" | jq -r ".releases[\"$SOLC_VERSION\"]")

    if [ "$bin" = "null" ]; then
        echo "Invalid Solc version: $SOLC_VERSION" 1>&2
        exit 1
    fi

    # windows versions <= 0.7.1 use .zip
    if [[ "$bin" = *.zip ]]; then
        echo "Cannot install solc <= 0.7.1" 1>&2
        exit 1
    fi

    curl -so "$bin" "$bins_url/$bin"
    mv -f "$bin" "solc$EXT"
    chmod +x "solc$EXT"
}

main
