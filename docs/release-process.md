
# Onion Release Process

## Versioning

Onion uses semantic-style versions:

MAJOR.MINOR.PATCH

During the experimental 0.x development series,
breaking changes may occur between minor versions.

Example:
- 0.1.0: Initial experimental release
- 0.1.1: Bug fixes
- 0.2.0: New language features

## Release Checklist

Before creating a release:

1. Confirm the intended version in Cargo.toml.
2. Review CHANGELOG.md.
3. Run cargo fmt --check.
4. Run cargo check.
5. Run cargo test.
6. Run repository smoke tests.
7. Build release binaries.
8. Verify the compiler version.
9. Test the compiler in a clean Windows environment.
10. Build and test the VS Code extension.
11. Prepare release archives and checksums.
12. Publish the GitHub release.
13. Update installation documentation.

## Test Commands

Run these commands from the repository root:

```powershell
cd onion-compiler

cargo fmt --check
cargo check --all-targets
cargo test
cargo build --release --bins

cd ..
./scripts/check.ps1
```

## Required Artifacts

The Windows release should contain:

- onion.exe
- onion-lsp.exe
- A portable installation script
- The VS Code extension VSIX
- LICENSE
- Installation instructions
- Checksums

## Security

Do not publish builds with known unresolved
critical security problems.

Release archives should be built from a reviewed
Git commit and checked against the corresponding tag.

## Publishing

Once testing succeeds, create a Git tag for the
release and publish binaries through GitHub Releases.

Do not create a public release from an untested
development build.
