<div align="center">

<img src="./onion-compiler/vscode-onion/icons/onion.png" alt="Onion Programming Language" width="150">

# The Onion Programming Language

**A Rust-inspired programming language, built from the ground up.**

[GitHub](https://github.com/RivalAkmals/onion-lang) | [Getting Started](#getting-started) | [Documentation](#documentation) | [Contributing](#contributing)

</div>

---

This is the main source code repository for **Onion**, an experimental programming language implemented in Rust.

The repository contains the Onion compiler, language server, Visual Studio Code integration, and example programs.

Onion takes inspiration from Rust's syntax and development experience while being developed as an independent language project.

The language and its tooling are under active development. The compiler is not yet considered stable or production-ready.

## Why Onion?

Onion is being developed around three main goals:

- **Familiarity:** A Rust-inspired syntax with familiar functions, variables, expressions, and control-flow constructs.

- **Developer experience:** Compiler diagnostics, editor integration, syntax highlighting, and language-server support developed alongside the compiler.

- **Extensibility:** A compiler architecture that can evolve to support additional language features, tooling, and platforms.

Onion is an independent project. It is not a fork of the Rust compiler and does not currently offer all of Rust's language features or guarantees.

## Getting Started

An Onion program is stored in a `.onion` file. The shorter `.oni` extension is also supported.

A minimal program looks like this:

```rust
fn main() {
    println!("Hello, Onion!");
}
```

Save the program as `main.onion`.

Once Onion is installed and available on your system's PATH, run:

```powershell
onion run main.onion
```

Output:

```text
Hello, Onion!
```

For now, Onion is primarily developed and tested on Windows.

## Installing from Source

Onion is currently distributed as a source project while the public release and installation process is being developed.

### Requirements

- [Rust and Cargo](https://rustup.rs/)
- [Git](https://git-scm.com/)
- [Node.js](https://nodejs.org/) (for building the VS Code extension)
- A Windows development environment

### Clone the repository

```bash
git clone https://github.com/RivalAkmals/onion-lang.git
cd onion-lang
```

### Build the compiler

```powershell
cd onion-compiler
cargo build --release --bins
```

The resulting binaries are located in:

```text
target/release/onion.exe
target/release/onion-lsp.exe
```

The `onion` executable provides the compiler's command-line interface. The `onion-lsp` executable provides Language Server Protocol integration.

To use the compiler from another directory, add the binary directory to your PATH.

## Editor Support

Onion includes a Visual Studio Code extension located in [`onion-compiler/vscode-onion`](./onion-compiler/vscode-onion).

The extension provides:

- Syntax highlighting for `.onion` and `.oni` files
- Language identification and file icons
- Compiler diagnostics in the editor
- Code completion
- Language Server Protocol integration

### Building the extension

From the repository root:

```powershell
cd onion-compiler/vscode-onion
npm install
npx @vscode/vsce package
```

Install the generated `.vsix` file through Visual Studio Code using the **Extensions: Install from VSIX** command.

The extension requires the Onion language server. Its location can be configured through the `onion.serverPath` setting or resolved through your system's PATH.

## Repository Structure

The source tree currently contains the following main components:

```text
onion/
├── onion-compiler/       # Compiler and language server
│   ├── src/              # Rust source code
│   ├── vscode-onion/     # Visual Studio Code extension
│   ├── Cargo.toml        # Rust package manifest
│   └── Cargo.lock        # Dependency lockfile
│
├── hello-onion/          # Example Onion programs
├── install.ps1           # Windows installation script
└── README.md
```

The project structure will grow as the language implementation, tests, documentation, standard library, and build tooling expand.

## Compiler Development

The Onion compiler is written in Rust.

Development currently focuses on language parsing, execution and code generation, compiler errors, and editor tooling.

To build the compiler in development mode:

```powershell
cd onion-compiler
cargo build
```

To check the Rust source code:

```powershell
cargo check
```

The compiler and language server are developed together to support both command-line compilation and interactive editor features.

## Documentation

Language documentation is still being prepared.

Planned documentation includes:

- Language syntax and basic concepts
- Variables, types, and expressions
- Functions and control flow
- Compiler usage and command-line options
- Standard library reference
- Editor configuration
- Compiler internals and contributor guides

An official documentation website is planned as part of the Onion project.

Until then, the repository and example source files serve as the primary development references.

## Project Status

Onion is currently in **early development**.

The compiler, language syntax, command-line interface, and tooling are subject to change.

The current development priorities are:

- [x] Initial compiler implementation
- [x] Basic Rust-inspired syntax
- [x] Command-line compiler
- [x] Language server foundation
- [x] Visual Studio Code extension
- [x] Syntax highlighting and diagnostics
- [ ] Expanded type system
- [ ] Additional language features
- [ ] Improved compiler diagnostics
- [ ] Standard library development
- [ ] Automated compiler test suite
- [ ] Code formatting and editor refactoring
- [ ] Public binary releases
- [ ] Cross-platform support
- [ ] Official installation tool
- [ ] Official language documentation
- [ ] Onion website

This roadmap is subject to change as the project develops.

## Website

An official Onion website is planned.

It will provide language documentation, installation instructions, compiler downloads, release information, and resources for developers.

The website will serve as the main entry point for users who want to install and learn Onion without building the compiler from source.

**Website: Coming soon.**

## Getting Help

If you encounter a compiler error, unexpected behavior, or an issue with the Visual Studio Code extension, please report it through [GitHub Issues](https://github.com/RivalAkmals/onion-lang/issues).

When reporting a compiler issue, include the Onion source code needed to reproduce it, the command you ran, and the resulting error message.

## Contributing

Contributions to Onion are welcome.

There are several areas where contributors can help, including compiler development, language tooling, testing, documentation, and platform support.

Before making significant changes, please open an issue to discuss the proposed implementation.

If you would like to contribute:

1. Fork the repository.
2. Create a branch for your changes.
3. Make and test your changes.
4. Submit a pull request.

Contributor documentation will be expanded as the project matures.

## License

Onion is intended to be distributed under the terms of the MIT License.

See the repository's `LICENSE` file once included.

---

<div align="center">

**Onion Programming Language**

*Built layer by layer.*

[Source Code](https://github.com/RivalAkmals/onion-lang) · [Issues](https://github.com/RivalAkmals/onion-lang/issues)

</div>
