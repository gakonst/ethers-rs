#!/usr/bin/env bash
# Installs Solc and Geth binaries
# Note: intended for use only with CI (x86_64 Ubuntu, MacOS or Windows)
set -e

GETH_BUILD=${GETH_BUILD:-"1.11.2-73b01f40"}
SOLC_VERSION=${SOLC_VERSION:-"0.8.19"}

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
    install_solc &
    wait

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
            GETH_ARCHIVE_NAME="geth-$PLATFORM-amd64-$GETH_BUILD"
            curl "https://gethstore.blob.core.windows.net/builds/$GETH_ARCHIVE_NAME.tar.gz" | tar -xzvf -
            mv -f "$GETH_ARCHIVE_NAME/geth" ./
            rm -rf "$GETH_ARCHIVE_NAME"
            chmod +x geth
            ;;
        *)
            GETH_ARCHIVE_NAME="geth-windows-amd64-$GETH_BUILD"
            zip="$GETH_ARCHIVE_NAME.zip"
            curl -o "$zip" "https://gethstore.blob.core.windows.net/builds/$zip"
            unzip "$zip"
            mv -f "$GETH_ARCHIVE_NAME/geth.exe" ./
            rm -rf "$GETH_ARCHIVE_NAME" "$zip"
            ;;
    esac
}

# Install the solc binary from the Github releases page
install_solc() {
    case "$PLATFORM" in
        linux)  SOLC_NAME="solc-static-linux";;
        darwin) SOLC_NAME="solc-macos";;
        *)      SOLC_NAME="solc-windows.exe";;
    esac
    curl -o "$SOLC_NAME" "https://github.com/ethereum/solidity/releases/download/v$SOLC_VERSION/$SOLC_NAME"
    mv -f "$SOLC_NAME" "solc$EXT"
    chmod +x "solc$EXT"
}

main
