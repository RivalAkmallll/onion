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
