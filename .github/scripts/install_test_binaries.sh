#!/usr/bin/env bash
# Installs Solc and Geth binaries
set -e

GETH_BUILD=${GETH_BUILD:-"1.11.2-73b01f40"}
SOLC_VERSION=${SOLC_VERSION:-"0.8.19"}

DIR="$HOME/bin"
mkdir -p "$DIR"
cd "$DIR"
export PATH="$DIR:$PATH"

echo "Installing Geth"
PLATFORM="$(uname -s | awk '{print tolower($0)}')"
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

geth version

echo "Installing Solc"
cargo install --locked svm-rs
# install only if it doesn't exist already
if command -v solc; then
    if [ -z "$(solc --version | grep "$SOLC_VERSION" || true)" ]; then
        svm install "$SOLC_VERSION"
    fi
    svm use "$SOLC_VERSION"
fi

solc --version
