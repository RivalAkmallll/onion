
# Security Policy

## Project Status

Onion is an experimental programming language.

The compiler and runtime are under active development.
No stable security guarantees are provided at this stage.

Onion should not currently be used for security-critical
or production systems.

## Supported Versions

| Version | Security Support |
|---------|------------------|
| 0.1.0  | Experimental, best-effort fixes |
| Earlier | Unsupported |

## Reporting a Vulnerability

Please do not publicly disclose a suspected vulnerability
before maintainers have had an opportunity to investigate it.

Use GitHub's private vulnerability reporting feature,
if enabled, through the repository's Security tab.

If private reporting is unavailable, open a GitHub issue
requesting a private reporting channel. Do not include
exploit details or sensitive information in that issue.

A useful report includes:

- A description of the vulnerability
- Affected Onion versions
- Steps to reproduce
- Expected and observed behavior
- Potential security impact
- A minimal reproduction, when safe to provide

## Security Goals

Onion aims to develop the following protections:

1. Static ownership and borrowing checks
2. Bounds-checked memory access
3. Explicit handling of potentially unsafe operations
4. Scoped allocation with validated lifetimes
5. Thread-safe concurrency primitives
6. Strict input validation in compiler tooling
7. Resource limits in network services
8. Dependency vulnerability monitoring
9. Automated compiler regression tests
10. Security-focused fuzz testing

These are development goals, not claims that all
protections are currently implemented.

## Disclosure Process

Security reports will be reviewed and investigated.

Confirmed vulnerabilities should receive a fix and
appropriate regression tests where practical.

The project may publish a GitHub Security Advisory
to document affected versions and mitigations.

As a small experimental project, Onion cannot
guarantee specific response or resolution times.

## Scope

Relevant security issues may include:

- Compiler crashes from malformed source code
- Memory corruption in generated programs
- Type-checker soundness problems
- Unsafe code generation
- Path traversal or unintended file overwrites
- Unsafe linking or temporary-file handling
- Language server vulnerabilities
- Dependency vulnerabilities
- Incorrect isolation of future runtime features

## Security Disclaimer

Passing compiler tests does not prove that Onion
programs are secure.

Applications written in Onion remain responsible for
input validation, access control, cryptographic
correctness, and other application-level protections.
