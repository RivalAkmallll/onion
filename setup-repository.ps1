
# ============================================
# Onion - Repository Development Setup
# Creates examples, tests, docs and CI.
# Does not overwrite existing project files.
# ============================================

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot

function Add-OnionFile {
    param(
        [string]$RelativePath,
        [string]$Content
    )

    $Path = Join-Path $Root $RelativePath
    $Parent = Split-Path $Path -Parent

    New-Item -ItemType Directory -Path $Parent -Force |
        Out-Null

    if (Test-Path $Path) {
        Write-Host "[SKIP] $RelativePath already exists" -ForegroundColor Yellow
        return
    }

    $Encoding = New-Object System.Text.UTF8Encoding($false)

    [System.IO.File]::WriteAllText(
        $Path,
        $Content.TrimStart("`r", "`n") + "`n",
        $Encoding
    )

    Write-Host "[CREATED] $RelativePath" -ForegroundColor Green
}

# ============================================
# EXAMPLES
# ============================================

Add-OnionFile "examples/hello-world.onion" @'
fn main() {
    println!("Hello, Onion!");
}
'@

Add-OnionFile "examples/counter.onion" @'
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
'@

# ============================================
# SMOKE TEST
# ============================================

Add-OnionFile "tests/smoke.ps1" @'
param(
    [string]$CompilerPath
)

$ErrorActionPreference = "Stop"

$Root = Split-Path $PSScriptRoot -Parent

if (-not $CompilerPath) {
    $CompilerPath = Join-Path $Root "onion-compiler/target/debug/onion.exe"
}

$CompilerPath = [System.IO.Path]::GetFullPath($CompilerPath)

if (-not (Test-Path $CompilerPath)) {
    throw "Onion compiler not found: $CompilerPath"
}

$Example = Join-Path $Root "examples/hello-world.onion"

Write-Host "Running Onion smoke test..." -ForegroundColor Cyan

$Output = & $CompilerPath run $Example 2>&1 | Out-String
$ExitCode = $LASTEXITCODE

Write-Host $Output

if ($ExitCode -ne 0) {
    throw "Onion compiler exited with code $ExitCode"
}

if ($Output -notmatch [regex]::Escape("Hello, Onion!")) {
    throw "Expected output was not found."
}

Write-Host "PASS: Hello World" -ForegroundColor Green
'@

# ============================================
# DEVELOPMENT SCRIPT
# ============================================

Add-OnionFile "scripts/check.ps1" @'
$ErrorActionPreference = "Stop"

$Root = Split-Path $PSScriptRoot -Parent
$CompilerDir = Join-Path $Root "onion-compiler"

Push-Location $CompilerDir

try {
    Write-Host "Checking Onion compiler..." -ForegroundColor Cyan

    & cargo check --all-targets

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo check failed."
    }

    Write-Host "Building Onion..." -ForegroundColor Cyan

    & cargo build --bins

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed."
    }

    Write-Host "Running Rust tests..." -ForegroundColor Cyan

    & cargo test

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo tests failed."
    }
}
finally {
    Pop-Location
}

$TestFile = Join-Path $Root "tests/smoke.ps1"

& $TestFile

if (-not $?) {
    throw "Onion smoke test failed."
}

Write-Host "All Onion checks passed!" -ForegroundColor Green
'@

# ============================================
# GITHUB ACTIONS
# ============================================

Add-OnionFile ".github/workflows/ci.yml" @'
name: Onion CI

on:
  push:
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

jobs:
  compiler:
    name: Build and Test Onion
    runs-on: windows-latest

    defaults:
      run:
        shell: pwsh

    steps:
      - name: Checkout repository
        uses: actions/checkout@v7

      - name: Check Rust version
        run: |
          rustc --version
          cargo --version

      - name: Check compiler
        working-directory: onion-compiler
        run: cargo check --all-targets

      - name: Build compiler
        working-directory: onion-compiler
        run: cargo build --bins

      - name: Run Rust tests
        working-directory: onion-compiler
        run: cargo test

      - name: Run Onion smoke test
        run: ./tests/smoke.ps1
'@

# ============================================
# LANGUAGE DOCUMENTATION
# ============================================

Add-OnionFile "docs/language-basics.md" @'
# Onion Language Basics

This document introduces the early Onion language.

Onion is experimental, and its syntax may change.

## Source files

Onion supports two source file extensions:

- `.onion`
- `.oni`

## Program entry point

A basic Onion program defines a `main` function.

```rust
fn main() {
    println!("Hello, Onion!");
}
```

## Variables

Variables are introduced with `let`.

```rust
let counter = 0
```

Mutable variables use `let mut`.

```rust
let mut counter = 0
counter += 1
```

## Functions

Functions are declared using `fn`.

```rust
fn main() {
    println!("Hello!");
}
```

## Conditional statements

Onion supports conditional control flow.

```rust
fn main() {
    let value = 10

    if value > 5 {
        println!("Greater than five")
    } else {
        println!("Five or less")
    }
}
```

## Loops

The `loop` construct repeats a block.

```rust
fn main() {
    let mut counter = 0

    loop {
        counter += 1

        if counter >= 5 {
            break
        }
    }
}
```

## Output

Use `println!` to print output.

```rust
fn main() {
    println!("Hello, Onion!");
}
```

## Language status

The examples here describe basic syntax used during
development. They are not a complete language specification.

Supported features will be documented as the compiler evolves.
'@

# ============================================
# DEVELOPER DOCUMENTATION
# ============================================

Add-OnionFile "docs/development.md" @'
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
'@

# ============================================
# CONTRIBUTING GUIDE
# ============================================

Add-OnionFile "CONTRIBUTING.md" @'
# Contributing to Onion

Thanks for your interest in Onion.

Onion is an experimental programming language. Contributions
to the compiler, tests, documentation, and editor tooling
are welcome.

## Before contributing

Check existing issues and pull requests before starting
a significant change.

For larger features, open an issue to discuss the design.

## Development setup

Install Rust and Cargo, then clone the repository.

Build the compiler:

```powershell
cd onion-compiler
cargo build --bins
```

Run tests:

```powershell
cargo test
```

## Pull requests

Keep changes focused and explain what they change.

For compiler bugs, include a small Onion program that
reproduces the original problem.

Add or update tests when changing compiler behavior.

Avoid unrelated formatting changes.

## Coding guidelines

- Follow existing Rust conventions.
- Prefer clear error messages.
- Keep compiler components maintainable.
- Document new language behavior.
- Do not introduce breaking syntax changes without discussion.

## Reporting bugs

Include:

1. The Onion source file or a minimal example.
2. The command used.
3. Expected behavior.
4. Actual behavior and error output.

## Project status

Onion is still early in development. Features and internal
interfaces may change.
'@

# ============================================
# INSTALL DOCUMENTATION
# ============================================

Add-OnionFile "INSTALL.md" @'
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
'@

# ============================================
# EDITORCONFIG
# ============================================

Add-OnionFile ".editorconfig" @'
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
indent_style = space
indent_size = 4
trim_trailing_whitespace = true

[*.{yml,yaml,json}]
indent_size = 2

[*.md]
trim_trailing_whitespace = false
'@

# ============================================
# GIT ATTRIBUTES
# ============================================

Add-OnionFile ".gitattributes" @'
*.rs text eol=lf
*.onion text eol=lf
*.oni text eol=lf
*.md text eol=lf
*.json text eol=lf
*.yml text eol=lf
*.yaml text eol=lf
*.ps1 text eol=crlf

*.png binary
*.jpg binary
*.ico binary
*.exe binary
*.vsix binary
'@

Write-Host ""
Write-Host "====================================" -ForegroundColor Green
Write-Host "  ONION REPOSITORY SETUP COMPLETE" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green
Write-Host ""
Write-Host "Existing compiler files were preserved."
Write-Host "Existing repository files were not overwritten."
Write-Host ""
Write-Host "Next: Run ./scripts/check.ps1"
