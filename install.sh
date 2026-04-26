#!/bin/bash
set -e

echo "Building nyado in release mode..."
cargo build --release

echo "Installing binary to ~/.local/bin/"
mkdir -p ~/.local/bin
cp target/release/nyado ~/.local/bin/

echo "Installing config files to ~/.config/nyado/"
mkdir -p ~/.config/nyado
cp config/*.toml ~/.config/nyado/

echo "Done! Run 'nyado' to start."