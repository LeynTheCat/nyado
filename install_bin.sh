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
        sudo apt install -y curl
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
BIN_URL="https://github.com/$REPO/releases/latest/download/nyado"

print_green "Downloading latest nyado binary from $BIN_URL"
curl -L -o nyado "$BIN_URL"
chmod +x nyado

print_green "Installing to ~/.local/bin/"
mkdir -p "$HOME/.local/bin"
mv nyado "$HOME/.local/bin/"

if [ ! -d "config" ]; then
    print_yellow "config folder not found. Fetching it from GitHub..."
    TMP_DIR=$(mktemp -d)
    git clone --depth=1 "https://github.com/$REPO.git" "$TMP_DIR"
    cp -r "$TMP_DIR/config" .
    rm -rf "$TMP_DIR"
    print_green "Config files downloaded."
fi

print_green "Removing old config files from ~/.config/nyado/"
rm -rf "$HOME/.config/nyado"
mkdir -p "$HOME/.config/nyado"

print_green "Installing fresh config files to ~/.config/nyado/"
cp config/*.toml "$HOME/.config/nyado/"

print_green "Nyado installed successfully."
print_green "Run 'nyado' to start."

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    print_yellow "Note: $HOME/.local/bin is not in your PATH. Add it to your shell config."
fi