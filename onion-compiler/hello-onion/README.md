# Onion Programming Language 🧅

**Onion** is an experimental, Rust-inspired programming language built with Rust.

Onion aims to provide familiar, expressive syntax with a dedicated compiler, language server, and Visual Studio Code extension.

## Features

- Rust-inspired syntax
- `.onion` and `.oni` source files
- Native Windows compiler
- Visual Studio Code syntax highlighting
- Language Server Protocol (LSP) integration
- Error diagnostics and code completion

## Example

```rust
fn main() {
    println!("Hello, Onion!");
}
```

## Getting Started

Download the latest Windows release from the GitHub Releases page.

Install the Onion compiler and the VS Code extension included in the release, then create a file named `main.onion`.

## Development

The compiler is built using Rust and Cargo.

```powershell
cargo build --release
```

The Visual Studio Code extension is located in the `vscode-onion` directory.

## Project Status

Onion is in early development. Its syntax, APIs, compiler features, and tooling may change.

## License

MIT, subject to the repository's included license file.
