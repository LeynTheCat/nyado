#!/bin/bash
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_green() { echo -e "${GREEN}$1${NC}"; }
print_yellow() { echo -e "${YELLOW}$1${NC}"; }

if [ "$(uname -s)" != "Linux" ]; then
    print_yellow "Warning: This script is designed for Linux. Your OS: $(uname -s)"
fi

if ! command -v curl &> /dev/null; then
    print_yellow "curl not found. Installing..."
    if command -v apt &> /dev/null; then
        sudo apt update && sudo apt install -y curl
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm curl
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y curl
    else
        print_yellow "Please install curl manually and rerun."
        exit 1
    fi
fi

REPO="LeynTheCat/nyado"
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        BIN_NAME="nyado-x86_64-unknown-linux-musl"
        ;;
    aarch64)
        BIN_NAME="nyado-aarch64-unknown-linux-musl"
        ;;
    *)
        print_yellow "Unsupported architecture: $ARCH. Trying generic 'nyado' (may fail)."
        BIN_NAME="nyado"
        ;;
esac

LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep -o '"tag_name": "[^"]*"' | cut -d '"' -f 4)
if [ -z "$LATEST_TAG" ]; then
    print_yellow "Could not detect latest tag, using 'latest'"
    BIN_URL="https://github.com/$REPO/releases/latest/download/$BIN_NAME"
else
    BIN_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$BIN_NAME"
fi

print_green "Downloading latest nyado binary for $ARCH from $BIN_URL"
curl -L -o nyado "$BIN_URL"
chmod +x nyado

print_green "Installing to ~/.local/bin/"
mkdir -p "$HOME/.local/bin"
mv nyado "$HOME/.local/bin/nyado"

fetch_config() {
    print_yellow "Downloading config files from GitHub..."
    if [ -z "$LATEST_TAG" ]; then
        ARCHIVE_URL="https://github.com/$REPO/archive/refs/heads/main.tar.gz"
    else
        ARCHIVE_URL="https://github.com/$REPO/archive/refs/tags/$LATEST_TAG.tar.gz"
    fi
    TMP_DIR=$(mktemp -d)
    curl -L -o "$TMP_DIR/repo.tar.gz" "$ARCHIVE_URL"
    tar -xzf "$TMP_DIR/repo.tar.gz" -C "$TMP_DIR"

    CONFIG_SRC=$(find "$TMP_DIR" -type d -name "config" | head -n1)
    if [ -n "$CONFIG_SRC" ]; then
        cp -r "$CONFIG_SRC" .
        print_green "Config files downloaded."
    else
        print_yellow "Config folder not found in archive."
    fi
    rm -rf "$TMP_DIR"
}

if [ ! -d "config" ]; then
    fetch_config
fi

print_green "Removing old config files from ~/.config/nyado/"
rm -rf "$HOME/.config/nyado"
mkdir -p "$HOME/.config/nyado"

print_green "Installing fresh config files to ~/.config/nyado/"
cp config/*.toml "$HOME/.config/nyado/"

print_green "Nyado installed successfully."
print_green "Run 'nyado' to start."

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    print_yellow "Note: $HOME/.local/bin is not in your PATH."
    echo -n "Do you want to add it to your shell configuration (recommended)? [y/N]: "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        SHELL_NAME=$(basename "$SHELL")
        if [ "$SHELL_NAME" = "bash" ]; then
            RC_FILE="$HOME/.bashrc"
        elif [ "$SHELL_NAME" = "zsh" ]; then
            RC_FILE="$HOME/.zshrc"
        elif [ "$SHELL_NAME" = "fish" ]; then
            RC_FILE="$HOME/.config/fish/config.fish"
            echo "set -gx PATH \$PATH $HOME/.local/bin" >> "$RC_FILE"
            print_green "Added to $RC_FILE (fish). Please restart your shell."
            exit 0
        else
            RC_FILE="$HOME/.profile"
        fi
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$RC_FILE"
        print_green "Added to $RC_FILE. Please restart your shell or run: source $RC_FILE"
    else
        print_yellow "You can manually add '$HOME/.local/bin' to your PATH later."
    fi
fi