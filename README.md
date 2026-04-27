# nyado – a Rust todo‑list with TUI

![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

nyado is a terminal-based task manager inspired by meowdo.
It supports multiple languages, tags, search, pinning, due dates.

![nyado preview](img/preview.png)

## Installation

Choose one of the following methods:

### 1. Quick install (binary, no compilation)

~~~
curl -sSL https://raw.githubusercontent.com/LeynTheCat/nyado/main/install_bin.sh | bash
~~~

This script:
- Detects your CPU architecture (x86_64 or aarch64)
- Downloads the latest pre‑built static binary from GitHub Releases
- Installs it to ~/.local/bin/
- Fetches and installs language files to ~/.config/nyado/ (replaces old configs)

### 2. Build from source (requires Rust)

~~~
curl -sSL https://raw.githubusercontent.com/LeynTheCat/nyado/main/install.sh | bash
~~~

The script will:
- Download the latest source code from GitHub
- Install Rust/Cargo automatically (Arch, Debian/Ubuntu, Fedora, openSUSE, or rustup)
- Build nyado in release mode
- Install binary and config files

### 3. Manual installation (git clone)

~~~
git clone https://github.com/LeynTheCat/nyado.git
cd nyado
./install.sh
~~~

or without cloning:

~~~
cargo install --git https://github.com/LeynTheCat/nyado.git
mkdir -p ~/.config/nyado
cp config/*.toml ~/.config/nyado/
~~~

## Update

- **Binary installation**: simply run the same quick install command again – it will download the latest binary and update config files.
- **Source installation (from git)**: cd into the cloned directory and run `./install.sh update`.
- **If you used the one‑line curl installer**: just run the same command again – it will overwrite the binary and configs.

## Uninstall

To completely remove nyado:

~~~
./install.sh uninstall
~~~

This deletes the binary from ~/.local/bin/ and the config directory ~/.config/nyado/.
Your tasks data is stored separately in ~/.local/share/nyado/ – if you want to remove that too, delete it manually:

~~~
rm -rf ~/.local/share/nyado
~~~

## Usage

Just run `nyado` from your terminal.

### Key bindings

Action              | Keys (English / Russian)
--------------------|-----------------------------------------
Quit                | q / й
Language switch     | l / L / л / Л
Navigate down       | j / о , ↓
Navigate up         | k / к , ↑
Top / Bottom        | g / г , G / Г (or Home / End)
Page down / up      | PageDown / PageUp
New task            | n / т
Edit task           | e / у
Toggle done         | Space
Pin / unpin         | p / з
Set tag             | t / е
Delete task         | d / в
Delete all tasks    | D / В (Shift + letter)
Search              | / / .
Filter by tag (1‑9) | 1…9 (only for existing tags)
Clear filters       | Esc
Set due date/time   | M / m / ь / Ь

Note: Filtering works for the first nine most‑used tags displayed in the right panel.
Press 1‑9 to filter by that tag, press Esc to clear the filter and the search query.

## Localisation

- Language files are stored in ~/.config/nyado/lang_*.toml (e.g., lang_en.toml, lang_ru.toml).
- You can add your own language by placing a lang_xx.toml file there (just copy an existing one and translate).
- The default language order is English, Russian, Chinese, Japanese, Spanish (determined by file names).

## Data storage

Tasks are saved in ~/.local/share/nyado/todos.txt in a simple pipe‑separated format.
You can back it up or edit manually (but be careful).

## Requirements

- Linux (x86_64 or aarch64) – any distribution with a decent terminal (unicode support).
- For the binary installer: curl.
- For the source installer: Rust toolchain (installed automatically if missing).

## Contributing

Feel free to open issues or pull requests.
The code is modular (each UI component lives in src/ui/), and the i18n system supports adding new languages easily.