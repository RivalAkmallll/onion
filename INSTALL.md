# Installing Onion

Onion is an experimental programming language.

## Public releases

Prebuilt Windows packages are planned.

Check the GitHub Releases page for published packages.

A public installer should not require users to install
Rust or Cargo.

## Development installation

To build Onion from source, install the Rust toolchain
and Cargo.

Clone this repository and run:

```powershell
cd onion-compiler
cargo build --release --bins
```

The resulting Windows executables are:

- target/release/onion.exe
- target/release/onion-lsp.exe

Add their directory to your PATH to use the compiler
from another terminal.

## Running a program

Create a source file named `main.onion`:

```rust
fn main() {
    println!("Hello, Onion!");
}
```

Run:

```powershell
onion run main.onion
```

## Editor integration

The VS Code extension is maintained under
`onion-compiler/vscode-onion`.

A public editor extension distribution is planned.

The extension requires the Onion language server,
either available on PATH or configured using
the `onion.serverPath` setting.
