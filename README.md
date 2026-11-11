<div align="center">

<img src="./onion-compiler/vscode-onion/icons/onion.png" alt="Onion Programming Language" width="160">

# The Onion Programming Language

**A Rust-inspired programming language, built from the ground up.**

[Website (Coming Soon)](#website) | [Getting Started](#getting-started) | [Documentation](#documentation) | [Contributing](#contributing)

</div>

---

This is the main source code repository for **Onion**, an experimental programming language implemented in Rust.

Onion is designed around familiar syntax, straightforward language constructs, and integrated developer tooling.

The repository contains the compiler, language server, editor integration, and the foundations of a growing programming language ecosystem.

Onion is under active development and is not yet recommended for production use.

## Why Onion?

Onion is being developed with three main goals in mind:

- **Familiarity:** A Rust-inspired syntax that makes functions, variables, expressions, and control flow recognizable to developers familiar with modern programming languages.

- **Simplicity:** A straightforward approach to writing and understanding programs, with an emphasis on readable syntax and useful compiler diagnostics.

- **Developer Experience:** A language that includes its own compiler, command-line tools, and editor integration rather than relying entirely on third-party tooling.

Onion is an independent programming language project. It is not a fork of Rust and does not claim to provide Rust's ownership model, memory-safety guarantees, or full language compatibility.

## Getting Started

An Onion source file uses the `.onion` extension. The shorter `.oni` extension is also supported.

A minimal Onion program looks like this:

```rust
fn main() {
    println!("Hello, Onion!");
}
```

Save the file as `main.onion`.

Once the Onion compiler is installed, run the program from your terminal:

```powershell
onion run main.onion
```

Output:

```text
Hello, Onion!
```

That's all you need to write and run a basic Onion program.

### Installing Onion

A public Windows installer and downloadable compiler releases are being prepared.

Once available, users will be able to download a prebuilt Onion compiler without installing Rust or building any source code.

Until the first public release, Onion is available through its source repository.

Visit [GitHub Releases](https://github.com/RivalAkmals/onion-lang/releases) to check for available downloads.

## The Language

Onion takes inspiration from Rust's syntax while being developed as its own language.

Here is an example of the control-flow syntax used in Onion:

```rust
fn main() {
    let mut counter = 0;

    loop {
        counter += 1;

        if counter % 2 == 0 {
            continue;
        }

        println!("counter = {}", counter);

        if counter >= 9 {
            break;
        }
    }

    println!("Finished!");
}
```

The language is still evolving. Not every Rust feature is supported, and future releases may introduce changes to the syntax and compiler behavior.

## Compiler and Tooling

The Onion project currently includes several core components.

| Component | Description |
|---|---|
| **Onion Compiler** | Processes and runs Onion source programs |
| **Onion CLI** | Command-line interface for working with Onion files |
| **Onion LSP** | Language Server Protocol implementation |
| **Editor Integration** | Syntax highlighting, diagnostics, and code completion |
| **Source File Support** | `.onion` and `.oni` file extensions |

Onion is currently developed primarily for Windows. Support for additional platforms is planned.

## Editor Support

Onion includes a dedicated Visual Studio Code extension that recognizes Onion source files.

It provides syntax highlighting, error diagnostics, language-server integration, and code completion.

The extension also includes the official Onion icon for supported file icon themes.

For regular users, the goal is to provide a ready-to-install extension through the Visual Studio Code Marketplace.

The extension does not need to be built manually once an official release is available.

## Repository Structure

The current repository contains the following primary components:

```text
onion/
├── onion-compiler/
│   ├── src/              # Compiler source code
│   ├── vscode-onion/     # Editor integration
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── hello-onion/          # Example programs
├── install.ps1           # Windows installation tooling
└── README.md
```

The repository will continue to expand as the compiler, standard library, tests, documentation, and additional developer tools are implemented.

## Documentation

Official Onion documentation is being prepared.

The planned documentation will cover language syntax, data types, functions, control flow, compiler commands, and standard library APIs.

The documentation will eventually be available through the official Onion website.

Until then, the repository and example programs provide an introduction to the language.

## Roadmap

Onion is in early development. Work is focused on improving the compiler, extending the language, and making Onion easier to install and use.

- [x] Initial compiler implementation
- [x] Rust-inspired syntax
- [x] Command-line compiler
- [x] Language Server Protocol foundation
- [x] Visual Studio Code integration
- [x] Syntax highlighting and diagnostics
- [ ] Expanded type system
- [ ] Improved compiler diagnostics
- [ ] Standard library
- [ ] Automated test suite
- [ ] Public Windows installer
- [ ] Prebuilt compiler downloads
- [ ] VS Code Marketplace release
- [ ] Linux and macOS support
- [ ] Official documentation
- [ ] Official Onion website
- [ ] Online playground

The roadmap reflects current development goals and may change as the project evolves.

## Website

An official Onion website is planned.

The website will provide a central place to download Onion, read documentation, explore language examples, and follow project releases.

The aim is to make installation and learning straightforward, without requiring users to clone the repository or understand the compiler's internal architecture.

**Official website: Coming soon.**

## Getting Help

If you encounter a compiler bug, unexpected behavior, or a problem with Onion tooling, please open an issue on GitHub.

When reporting an issue, include a small example that reproduces the problem, the command used, and the resulting error output.

[Report an Issue](https://github.com/RivalAkmals/onion-lang/issues)

## Contributing

Onion is an independent, open-source programming language project.

Contributions are welcome in compiler development, editor integration, testing, documentation, and tooling.

Developers interested in contributing can fork the repository, make their changes, and submit a pull request.

Contributor documentation and source-build instructions will be maintained separately from the main getting-started guide.

## License

Onion is intended to be distributed under the MIT License.

See `LICENSE` for the applicable terms once the license file is included in the repository.

---

<div align="center">

<img src="./onion-compiler/vscode-onion/icons/onion.png" alt="Onion" width="65">

**Onion Programming Language**

*Built layer by layer.*

[GitHub](https://github.com/RivalAkmals/onion-lang) | [Issues](https://github.com/RivalAkmals/onion-lang/issues) | [Releases](https://github.com/RivalAkmals/onion-lang/releases)

</div>
