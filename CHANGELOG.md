
# Changelog

All notable changes to Onion will be documented here.

Onion is currently experimental. The language,
compiler APIs, and tooling may change without
backward compatibility during the 0.x series.

## [Unreleased]

### Planned

- Ownership and borrow checking
- Scoped arena allocation
- Managed memory scopes
- Actor-style concurrency
- Native shader compilation
- HTTP standard library
- UI integration
- Expanded compiler security tests

## [0.1.0] - Unreleased

### Added

- Initial Onion compiler implemented in Rust
- Onion command-line interface
- `run`, `build`, and `check` commands
- Native code generation and linking
- Interpreter execution mode
- Lexer and parser
- Basic type checking
- Compiler error diagnostics
- Language server foundation
- Visual Studio Code extension
- `.oni` and `.onion` source file support

### Development

- Compiler regression tests
- Repository smoke tests
- Initial documentation
- Windows development tooling

### Known Limitations

- No complete ownership and borrowing model
- No stable standard library
- No native shader support
- No actor runtime
- No built-in HTTP framework
- No Onion-native UI backend
- Windows-focused implementation

The release date and final feature list will be
confirmed when v0.1.0 is published.
