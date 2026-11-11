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
