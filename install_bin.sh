#!/bin/bash
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_green() { echo -e "${GREEN}$1${NC}"; }
print_yellow() { echo -e "${YELLOW}$1${NC}"; }
print_error() { echo -e "${RED}$1${NC}"; }

detect_platform() {
    if [ -n "$TERMUX_VERSION" ] || [ -d "/data/data/com.termux" ]; then
        echo "termux"
    elif [ "$(uname -s)" = "Linux" ]; then
        echo "linux"
    elif [ "$(uname -s)" = "Darwin" ]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

PLATFORM=$(detect_platform)

if [ "$PLATFORM" = "termux" ]; then
    BIN_DIR="$PREFIX/bin"
    CONFIG_DIR="$PREFIX/etc/nyado"
else
    BIN_DIR="$HOME/.local/bin"
    CONFIG_DIR="$HOME/.config/nyado"
fi

install_pkg() {
    local pkg="$1"
    if command -v "$pkg" &> /dev/null; then
        return 0
    fi
    print_yellow "Installing $pkg..."
    if [ "$PLATFORM" = "termux" ]; then
        pkg install -y "$pkg"
    elif command -v apt &> /dev/null; then
        if command -v sudo &> /dev/null; then
            sudo apt update && sudo apt install -y "$pkg"
        else
            print_error "sudo not available. Please install $pkg manually (apt install $pkg)"
            exit 1
        fi
    elif command -v pacman &> /dev/null; then
        if command -v sudo &> /dev/null; then
            sudo pacman -S --noconfirm "$pkg"
        else
            print_error "sudo not available. Please install $pkg manually (pacman -S $pkg)"
            exit 1
        fi
    elif command -v dnf &> /dev/null; then
        if command -v sudo &> /dev/null; then
            sudo dnf install -y "$pkg"
        else
            print_error "sudo not available. Please install $pkg manually (dnf install $pkg)"
            exit 1
        fi
    else
        print_error "Cannot install $pkg automatically. Please install it manually and rerun."
        exit 1
    fi
}

if ! command -v curl &> /dev/null; then
    install_pkg curl
fi

REPO="LeynTheCat/nyado"
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        BIN_NAME="nyado-x86_64-unknown-linux-musl"
        ;;
    aarch64|arm64)
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

print_green "Installing to $BIN_DIR"
mkdir -p "$BIN_DIR"
mv nyado "$BIN_DIR/nyado"

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

print_green "Removing old config files from $CONFIG_DIR"
rm -rf "$CONFIG_DIR"
mkdir -p "$CONFIG_DIR"

print_green "Installing fresh config files to $CONFIG_DIR"
cp config/*.toml "$CONFIG_DIR/"

print_green "Nyado installed successfully."
print_green "Run 'nyado' to start."

add_to_path() {
    local shell_rc=""
    local shell_name=$(basename "$SHELL")
    if [ "$shell_name" = "bash" ]; then
        shell_rc="$HOME/.bashrc"
    elif [ "$shell_name" = "zsh" ]; then
        shell_rc="$HOME/.zshrc"
    elif [ "$shell_name" = "fish" ]; then
        shell_rc="$HOME/.config/fish/config.fish"
        echo "set -gx PATH \$PATH $BIN_DIR" >> "$shell_rc"
        print_green "Added to $shell_rc (fish). Please restart your shell."
        return
    else
        shell_rc="$HOME/.profile"
    fi
    echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$shell_rc"
    print_green "Added to $shell_rc. Please restart your shell or run: source $shell_rc"
}

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    print_yellow "$BIN_DIR is not in your PATH."
    echo -n "Do you want to add it to your shell configuration (recommended)? [y/N]: "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        add_to_path
    else
        print_yellow "You can manually add '$BIN_DIR' to your PATH later."
    fi
fi