# Developing Onion

Onion is implemented in Rust.

This document describes the current development workflow.

## Requirements

- Rust toolchain
- Cargo
- Git
- Windows for the current smoke-test workflow

## Compiler source

The compiler project is located in:

`onion-compiler/`

Its Rust source code is located in:

`onion-compiler/src/`

The existing source layout is intentionally preserved.

## Build

From the repository root:

```powershell
cd onion-compiler
cargo build --bins
```

## Run Rust tests

```powershell
cargo test
```

## Run repository checks

From the repository root:

```powershell
./scripts/check.ps1
```

This builds Onion, runs Rust tests, and checks a small
Onion program through the command-line compiler.

## Editor integration

The Visual Studio Code extension lives in:

`onion-compiler/vscode-onion/`

The language server is built alongside the compiler.

## Code organization

As the compiler grows, the lexer, parser, type checker,
code generator, and diagnostics can be separated into
dedicated Rust modules.

Changes to the module structure should preserve existing
compiler functionality.
