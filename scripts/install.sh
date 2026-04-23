#!/usr/bin/env bash
set -euo pipefail

REPO="LordSalmon/claudestine"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="claudestine"

detect_os() {
    case "$(uname -s)" in
        Darwin) echo "macos" ;;
        Linux)  echo "linux" ;;
        *)
            echo "Unsupported OS: $(uname -s)" >&2
            exit 1
            ;;
    esac
}

main() {
    local os
    os=$(detect_os)

    echo "Fetching latest release for $os..."

    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local release_json
    release_json=$(curl -fsSL "$api_url")

    local asset_name
    asset_name=$(echo "$release_json" \
        | grep -o '"name": *"[^"]*'"$os"'[^"]*"' \
        | head -1 \
        | sed 's/"name": *"\(.*\)"/\1/')

    if [[ -z "$asset_name" ]]; then
        echo "No release asset found for OS: $os" >&2
        exit 1
    fi

    local download_url
    download_url=$(echo "$release_json" \
        | grep -o '"browser_download_url": *"[^"]*'"$asset_name"'[^"]*"' \
        | head -1 \
        | sed 's/"browser_download_url": *"\(.*\)"/\1/')

    if [[ -z "$download_url" ]]; then
        echo "Could not find download URL for asset: $asset_name" >&2
        exit 1
    fi

    echo "Downloading $asset_name..."
    mkdir -p "$INSTALL_DIR"
    curl -fsSL "$download_url" -o "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    echo "Installed to $INSTALL_DIR/$BINARY_NAME"

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        echo "$INSTALL_DIR is not in your PATH. To add it, run:"
        echo ""
        case "$os" in
            macos)
                echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zprofile && source ~/.zprofile"
                ;;
            linux)
                echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
                ;;
        esac
        echo ""
    fi
}

main
