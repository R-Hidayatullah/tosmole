

# tosmole

**tosmole** is a Rust project designed to parse Tree of Savior file types—including XSM, XAC, IPF, IES—and provide a web interface for browsing game data and duplicates.

## Table of Contents

* [Supported File Types](#supported-file-types)
* [Features](#features)
* [Installation](#installation)
* [Usage](#usage)
* [Configuration](#configuration)
* [Contributing](#contributing)
* [License](#license)

## Supported File Types

* **XSM** – Animation files (WIP)
* **XAC** – Character & skeleton files
* **IPF** – Game archive files
* **IES** – Tabular data

## Features

* Parses IPF archives and generates a hierarchical folder tree
* Computes file statistics (unique files, total files)
* Parses duplicates from XML files (XAC, XSM, XPM, DDS)
* Serves a web interface using **Actix Web** and **Tera** templates
* Configurable game root path, address, and port via `paths.json`

## Installation

### Prerequisites

* Install [Rust](https://rustup.rs/) (recommended via **rustup**)
* Ensure `cargo` is in your system PATH

### Compile from Source

1. Clone the repository:

```bash
git clone https://github.com/R-Hidayatullah/tosmole
cd tosmole
```

2. Compile the project in release mode:

```bash
cargo build --release
```

The executable will be located in `target/release/tosmole`.

## Configuration

**tosmole** loads paths and server configuration from `paths.json`:

```json
{
    "game_root": "/path/to/TreeOfSavior",
    "address": "127.0.0.1",
    "port": 8080
}
```

* `game_root`: Path to the Tree of Savior installation
* `address` (optional): Server address (default: `127.0.0.1`)
* `port` (optional): Server port (default: `8080`)

The language folder is automatically derived as:

```
<game_root>/release/languageData/English
```

## Usage

Run **tosmole** with:

```bash
cargo run --release
```

Or directly execute the compiled binary:

```bash
./target/release/tosmole
```

The server will start at the configured address and port. Open your browser and navigate to:

```
http://127.0.0.1:8080
```

### Web Interface

* Browse folder trees of IPF archives
* View duplicate entries parsed from XML
* Access file statistics

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `feature/my-feature`
3. Make your changes and commit
4. Push to your fork and open a Pull Request

## License

This project is licensed under **GPL-3.0**.

---
