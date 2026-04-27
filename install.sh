#!/bin/bash
set -e

BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/nyado"
BINARY_NAME="nyado"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

REPO="LeynTheCat/nyado"

check_rust() {
    if command -v cargo &> /dev/null; then
        print_info "Rust/cargo already installed: $(cargo --version)"
        return 0
    fi
    print_warn "Rust/cargo not found."
    return 1
}

install_rust() {
    print_info "Installing Rust via system package manager..."
    if command -v pacman &> /dev/null; then
        sudo pacman -S --needed --noconfirm rustup cargo
        rustup default stable
    elif command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y cargo
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y cargo
    elif command -v zypper &> /dev/null; then
        sudo zypper install -y cargo
    else
        print_warn "Falling back to rustup official installer."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    if command -v cargo &> /dev/null; then
        print_info "Rust/cargo installed successfully."
    else
        print_error "Failed to install Rust. Please install manually from https://rustup.rs/"
        exit 1
    fi
}

fetch_source() {
    print_info "Downloading source code from GitHub..."
    LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep -o '"tag_name": "[^"]*"' | cut -d '"' -f 4)
    if [ -z "$LATEST_TAG" ]; then
        print_error "Could not detect latest release tag."
        exit 1
    fi
    ARCHIVE_URL="https://github.com/$REPO/archive/refs/tags/$LATEST_TAG.tar.gz"
    TMP_DIR=$(mktemp -d)
    curl -L -o "$TMP_DIR/source.tar.gz" "$ARCHIVE_URL"
    tar -xzf "$TMP_DIR/source.tar.gz" -C "$TMP_DIR"
    SOURCE_DIR=$(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d | head -n1)
    if [ -z "$SOURCE_DIR" ]; then
        print_error "Failed to extract source code."
        rm -rf "$TMP_DIR"
        exit 1
    fi
    cp -r "$SOURCE_DIR/." .
    rm -rf "$TMP_DIR"
    print_info "Source code downloaded."
}

build() {
    print_info "Building nyado in release mode..."
    cargo build --release
}

install() {
    print_info "Installing binary to $BIN_DIR/"
    mkdir -p "$BIN_DIR"
    cp target/release/$BINARY_NAME "$BIN_DIR/"

    print_info "Removing old config files from $CONFIG_DIR"
    rm -rf "$CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"

    print_info "Installing fresh config files to $CONFIG_DIR"
    cp config/*.toml "$CONFIG_DIR/"

    print_info "Done! Run '$BINARY_NAME' to start."
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        print_warn "$BIN_DIR not in PATH. Add: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

if [ "$1" = "update" ]; then
    if [ -d ".git" ]; then
        print_info "Updating nyado from git repository..."
        git pull --rebase
        check_rust || install_rust
        build
        install
        print_info "Update completed."
    else
        print_error "Update is only available when installed from a cloned git repository."
        print_error "Please reinstall using the same script without 'update' to get the latest version."
        exit 1
    fi
elif [ "$1" = "uninstall" ]; then
    print_info "Removing binary from $BIN_DIR/$BINARY_NAME"
    rm -f "$BIN_DIR/$BINARY_NAME"
    print_info "Removing config directory $CONFIG_DIR"
    rm -rf "$CONFIG_DIR"
    print_info "nyado has been uninstalled."
else
    if [ ! -d "config" ]; then
        fetch_source
    fi
    if ! check_rust; then
        install_rust
    fi
    build
    install
fi