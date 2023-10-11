# tosmole

**tosmole** is a Rust project designed to parse Tree of Savior file types, including XSM, XAC, IPF, and IES.

## Table of Contents

- [Supported File Types](#supported-file-types)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Supported File Types

- XSM
- XAC
- IPF (WIP)
- IES

## Installation

To get started with **tosmole**, you need to install Rust and compile the project using `rustup`.

### Prerequisites

Before you begin, make sure you have [rustup](https://rustup.rs/) installed. Rustup is the recommended way to manage
Rust toolchains.

### Installing Rust and Compiling

1. **Install Rust:**

   If you haven't already, install Rust and Cargo by following the instructions on
   the [official Rust website](https://www.rust-lang.org/tools/install).

   If you haven't already, install Rust with the nightly version using rustup:

   ```shell
   rustup default nightly
   rustup install nightly
   ```

2. **Clone the Repository:**

   ```shell
   git clone https://github.com/R-Hidayatullah/tosmole
   cd tosmole
    ````
3. **Compile The Project:**

   Build the project using Cargo:
    ```shell
   cargo build --release
    ```
   This will compile the project in release mode, and the executable will be available in the `target/release`
   directory.

### Usage

Once you've successfully compiled tosmole, you can use it to parse Tree of Savior file types.
To use **tosmole**, follow these instructions:

1. **Compile the Program:**

   First, make sure you have Rust installed, and the project is compiled using the provided code. If you haven't already
   compiled it, please refer to the [Installation](#installation) section in the README.

2. **Run the Program:**

   To execute the program, open your terminal and run the following command:

   ```shell
   tosmole <path_to_file.ipf> [index_number]
   ```

### Contributing

Contributions to tosmole are welcome! To contribute, please follow these guidelines:

- Fork the repository.
- Create a feature branch (e.g., feature/my-new-feature).
- Make your changes and commit them.
- Push to your fork and submit a pull request.

### License

This project is licensed under the GPL-3.0 license.