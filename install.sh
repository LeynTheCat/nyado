#!/bin/bash
# nyado installer/uninstaller/updater
# Usage: ./install.sh [install|update|uninstall]

set -e

BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/nyado"
REPO_DIR="$(pwd)"
BINARY_NAME="nyado"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_rust() {
    if command -v cargo &> /dev/null; then
        print_info "Rust/cargo already installed: $(cargo --version)"
        return 0
    else
        print_warn "Rust/cargo not found."
        return 1
    fi
}

install_rust() {
    print_info "Attempting to install Rust via system package manager..."
    if command -v pacman &> /dev/null; then
        print_info "Arch Linux detected. Installing rust via pacman..."
        sudo pacman -S --needed --noconfirm rustup cargo
        rustup default stable
    elif command -v apt-get &> /dev/null; then
        print_info "Debian/Ubuntu detected. Installing rust via apt..."
        sudo apt-get update
        sudo apt-get install -y cargo
    elif command -v dnf &> /dev/null; then
        print_info "Fedora detected. Installing rust via dnf..."
        sudo dnf install -y cargo
    elif command -v zypper &> /dev/null; then
        print_info "openSUSE detected. Installing rust via zypper..."
        sudo zypper install -y cargo
    else
        print_warn "No known package manager found. Falling back to rustup official installer."
        print_info "Installing rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    if command -v cargo &> /dev/null; then
        print_info "Rust/cargo installed successfully."
    else
        print_error "Failed to install Rust/cargo. Please install manually from https://rustup.rs/"
        exit 1
    fi
}

build() {
    print_info "Building nyado in release mode..."
    cargo build --release
}

install() {
    print_info "Installing binary to $BIN_DIR/"
    mkdir -p "$BIN_DIR"
    cp target/release/$BINARY_NAME "$BIN_DIR/"

    print_info "Installing config files to $CONFIG_DIR/"
    mkdir -p "$CONFIG_DIR"
    cp config/*.toml "$CONFIG_DIR/"

    print_info "Done! You can now run '$BINARY_NAME' from your terminal."
    echo "Make sure $BIN_DIR is in your PATH (usually it is). If not, add it to your shell config: export PATH=\"\$HOME/.local/bin:\$PATH\""
}

update() {
    print_info "Updating nyado from git repository..."
    git pull --rebase
    build
    install
    print_info "Update completed."
}

uninstall() {
    print_info "Removing binary from $BIN_DIR/$BINARY_NAME"
    rm -f "$BIN_DIR/$BINARY_NAME"
    print_info "Removing config files from $CONFIG_DIR"
    rm -rf "$CONFIG_DIR"
    print_info "nyado has been uninstalled."
}

case "$1" in
    update)
        update
        ;;
    uninstall)
        uninstall
        ;;
    install|""|*)
        if ! check_rust; then
            install_rust
        fi
        build
        install
        ;;
esac