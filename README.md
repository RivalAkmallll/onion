<div align="center">

<img src="./onion-compiler/vscode-onion/icons/onion.png" alt="Onion Programming Language" width="140">

# Onion

**A security-conscious programming language, built layer by layer.**

Experimental · Rust implementation · Compiler/CLI focus · Version 0.1.0

[Quick start](#quick-start) · [What works](#what-works-today) · [Security](#security-status) · [Roadmap](#roadmap) · [Contributing](#contributing)

</div>

## Read this first

Onion is an early-stage language with a lexer, parser, type and ownership checks,
interpreter, Windows x64 native backend, command-line tool, language-server
foundation, and VS Code extension work.

It is **not production-ready**. Its supported compiler subset is being hardened,
but language rules, native output behavior, APIs, and tooling can change. Do not
use it yet for systems that handle real users, payments, credentials, health data,
or other sensitive workloads.

Onion source code cannot currently build websites, desktop apps, games, Android
apps, or iOS apps. Those are planned as separately tested platform profiles; they
will not be advertised before their individual release gates are met.

## What works today

| Area | Current capability | Status |
| --- | --- | --- |
| Language | Functions, variables, integers, floats, booleans, strings, expressions, conditions, loops, break, continue, and printing | Implemented subset |
| Compiler | Lexer, parser, diagnostics, type checking, conservative String move/borrow checks | Implemented, expanding |
| Execution | Interpreter and experimental Windows x64/MSVC native code generation | Experimental |
| CLI | check, run, and build commands | Implemented |
| Tooling | LSP foundation and a VS Code extension | In development |
| Web boundary | onion-web Rust-hosted HTTP security middleware for future integration | Experimental Phase 1 |
| Desktop/UI | Dioxus-based Rust experiment | Research only |

## Quick start

### Requirements

- Rust and Cargo
- Windows development tools for the experimental native backend
- Git, if cloning the repository

Clone and test the compiler:

~~~powershell
git clone https://github.com/RivalAkmallll/onion.git
cd onion\onion-compiler
cargo test --locked
~~~

Create hello.onion:

~~~rust
fn main() {
    let greeting = "Hello, Onion!";
    println!("{}", greeting);
}
~~~

Check it without running it:

~~~powershell
cargo run -- check ..\hello.onion
~~~

Run it through the compiler:

~~~powershell
cargo run -- run ..\hello.onion
~~~

Use --interp to run the interpreter explicitly:

~~~powershell
cargo run -- run ..\hello.onion --interp
~~~

Build a native executable with an unused output path:

~~~powershell
cargo run -- build ..\hello.onion --output ..\hello.exe
~~~

The build command intentionally refuses to overwrite an existing output file.

## Security status

Security is a design goal, not a blanket guarantee. Current work includes:

- bounded compiler source, token, nesting, and expression complexity;
- conservative ownership and mutable-borrow diagnostics for the supported subset;
- checked String allocation/concatenation paths and native/interpreter parity tests;
- no-clobber native build outputs and safer temporary-file handling;
- locked dependency tests and local advisory scanning;
- an experimental web boundary with request limits, strict CORS defaults,
  same-origin state-change checks, security headers, and basic output escaping.

These controls do **not** mean an Onion application is automatically secure.
HTTPS does not stop SQL injection; HTML escaping does not replace authorization;
a compiler check does not secure secrets or deployment. Database access,
authentication, authorization, uploads, outbound network access, and an
Onion-language web runtime are not implemented yet.

For precise scope and open gates, read:

- [Compiler release gates](docs/COMPILER-RELEASE-GATES.md)
- [Compiler resource security](docs/COMPILER-RESOURCE-SECURITY.md)
- [String memory model](docs/STRING-MEMORY.md)
- [Supported borrow restrictions](docs/PHASE-18.3.2-BORROW-SAFETY.md)
- [Onion Web Phase 1](docs/ONION-WEB-PHASE-1.md)
- [Web security roadmap](docs/WEB-SECURITY-ROADMAP.md)
- [Platform production roadmap](docs/PLATFORM-PRODUCTION-ROADMAP.md)

## Testing

Run compiler tests and quality checks:

~~~powershell
cd onion-compiler
cargo test --locked --all-targets
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
~~~

Run the experimental web-boundary tests:

~~~powershell
cd onion-web
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
~~~

Onion-web is a Rust library and is not evidence that Onion programs can serve
websites yet.

## Repository layout

~~~text
onion/
├── onion-compiler/     Compiler, interpreter, native backend, LSP, VS Code work
├── onion-web/          Experimental Rust-hosted web security boundary
├── docs/               Design documents, security scope, and release gates
├── examples/           Example Onion programs
├── experiments/        Non-production UI and platform experiments
├── tests/              Onion-language test inputs
└── scripts/            Development and verification scripts
~~~

## Roadmap

The immediate priority is a narrow, reviewable compiler/CLI production candidate
for the declared Windows x64 scope. It needs fuzzing, platform tests, reproducible
artifacts, hosted CI evidence, dependency monitoring, and independent review.

Only after the compiler and language/runtime contracts are stable can Onion add
web handlers, database bindings, and a web application profile. Desktop, games,
Android, and iOS then require their own runtimes, test matrices, signing/update
processes, and independent security reviews.

See the [platform production roadmap](docs/PLATFORM-PRODUCTION-ROADMAP.md) for
the full staged plan and its non-negotiable release gates.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and
[SECURITY.md](SECURITY.md) first. Do not report security-sensitive findings in a
public issue unless the security policy says it is safe to do so.

## License

Onion is licensed under the [GNU AGPL v3.0](LICENSE).
