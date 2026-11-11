 <div align="center">

<img src="./onion-compiler/vscode-onion/icons/onion.png" alt="Onion Programming Language" width="165">

# Onion

**A new language. A familiar feeling.**

A Rust-inspired programming language built from scratch, one layer at a time.

[![Version](https://img.shields.io/badge/version-0.1.0-8c42ad?style=flat-square)](https://github.com/RivalAkmals/onion-lang)
[![Rust](https://img.shields.io/badge/built_with-Rust-f46623?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-experimental-e5a339?style=flat-square)](https://github.com/RivalAkmals/onion-lang)
[![Platform](https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square&logo=windows&logoColor=white)](https://github.com/RivalAkmals/onion-lang)

[Getting Started](#getting-started) · [Language](#the-language) · [Editor Support](#editor-support) · [Roadmap](#roadmap)

*Official website and public downloads are on the roadmap.*

</div>

---

## Meet Onion.

Onion is an experimental programming language written in Rust.

It takes inspiration from Rust's syntax and programming style while exploring its own direction. The idea is to build something approachable without giving up the control and flexibility that make systems programming interesting.

Onion isn't trying to replace Rust, C++, or anything else.

It's a language being built because building a language is interesting — and because there are plenty of ideas worth exploring.

The compiler, language server, and editor integration are developed together, rather than treating tooling as an afterthought.

**Still early. Already real.**

## The language

If you've written Rust before, parts of Onion will look familiar.

```rust
fn main() {
    println!("Hello, Onion!");
}
```

Onion uses `.onion` files, with `.oni` supported as a shorter alternative.

Here's a larger example showing the kind of syntax the project is exploring:

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

    println!("Done!");
}
```

The language is still evolving. Some features may change or behave differently between releases.

## What's inside?

| Component | Description |
|---|---|
| **Onion Compiler** | The language's compiler, implemented in Rust |
| **Onion LSP** | Language Server Protocol support for editor integration |
| **VS Code Extension** | Syntax highlighting, completions, and diagnostics |
| **Error Diagnostics** | Problems reported directly inside the editor |
| **Onion CLI** | Command-line interface for working with Onion programs |
| **Custom File Support** | `.onion` and `.oni` source files |

The goal is to make Onion feel like a complete programming environment, from writing your first line of code to building and running a project.

## Getting started

Onion is currently available as a source project while the public distribution process is being prepared.

### Prerequisites

For development, you'll need:

- [Rust and Cargo](https://rustup.rs/)
- [Visual Studio Code](https://code.visualstudio.com/)
- [Node.js](https://nodejs.org/) for the editor extension
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

After a successful build, the compiler and language server binaries can be found in:

```text
target/release/onion.exe
target/release/onion-lsp.exe
```

### Build the VS Code extension

```powershell
cd vscode-onion
npm install
npx @vscode/vsce package
```

Install the generated `.vsix` through Visual Studio Code's **Extensions: Install from VSIX** command.

Make sure `onion-lsp.exe` is available on your PATH or configured through the `onion.serverPath` setting.

### Write your first program

Create `main.onion`:

```rust
fn main() {
    println!("Hello from Onion!");
}
```

Run it:

```powershell
onion run main.onion
```

Expected output:

```text
Hello from Onion!
```

**Note:** The current Windows tooling is experimental. These steps describe the development workflow, not a finalized public installer.

## Editor support

Onion has its own Visual Studio Code extension.

The extension currently targets:

- Syntax highlighting for Onion files
- Automatic recognition of `.oni` and `.onion`
- Error reporting and diagnostics
- Code completions
- Language server integration
- Custom Onion file icons

The language server is built in Rust, allowing the compiler's understanding of Onion to be reused in the editor.

Editor support will continue to improve alongside the language.

## Project structure

```text
onion/
├── onion-compiler/
│   ├── src/
│   ├── vscode-onion/
│   │   ├── icons/
│   │   │   └── onion.png
│   │   ├── syntaxes/
│   │   ├── extension.js
│   │   └── package.json
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── hello-onion/
│   └── main.oni
│
├── install.ps1
└── README.md
```

The repository contains the compiler implementation, editor tooling, and example programs.

## Roadmap

Onion is being developed in stages. The current focus is getting the language and its tools into a reliable, usable state.

**Language and compiler**
- [x] Initial compiler implementation
- [x] Rust-inspired function syntax
- [x] Command-line compiler
- [ ] Expand the type system
- [ ] Improve compiler diagnostics
- [ ] Add more language features
- [ ] Improve code generation and optimization
- [ ] Standard library development

**Developer experience**
- [x] VS Code language extension
- [x] Syntax highlighting
- [x] Language server foundation
- [x] Code completion foundation
- [x] Custom file icon
- [ ] More intelligent autocomplete
- [ ] Go to definition
- [ ] Rename symbol support
- [ ] Code formatting
- [ ] Debugger integration

**Distribution and ecosystem**
- [ ] Portable Windows installer
- [ ] Public binary releases
- [ ] VS Code Marketplace publication
- [ ] Linux support
- [ ] macOS support
- [ ] Package management
- [ ] Official documentation
- [ ] Official Onion website
- [ ] Online playground

These are development goals, not guaranteed release commitments.

## An official home for Onion

The repository is just the beginning.

An official website is planned to bring everything together: downloads, documentation, language examples, release notes, and a place for people to follow development.

The long-term idea is a language website with the same kind of developer-first experience found in established language ecosystems.

No installation guides scattered across random pages. No guessing which binary to download.

Just a straightforward way to get Onion, learn it, and start building.

**Website: Coming soon.**

Until then, GitHub is the project's home.

## Contributing

Onion is an independent project in early development.

Ideas, bug reports, and contributions are welcome.

If you'd like to contribute, open an issue to discuss a feature or submit a pull request with your changes.

As the language grows, contribution guidelines and development documentation will be expanded.

## Project status

**Experimental — v0.1.0**

Onion is not production-ready.

The language specification, compiler behavior, supported features, and tooling are subject to change. Breaking changes should be expected during early development.

If you run into a problem, feel free to open an issue.

[Report an issue](https://github.com/RivalAkmals/onion-lang/issues) · [View source](https://github.com/RivalAkmals/onion-lang)

## License

MIT License. See the `LICENSE` file for details.

---

<div align="center">

<img src="./onion-compiler/vscode-onion/icons/onion.png" alt="Onion" width="65">

**Onion**

*Built layer by layer.*

Created by [Rival Akmal](https://github.com/RivalAkmals)

</div>
