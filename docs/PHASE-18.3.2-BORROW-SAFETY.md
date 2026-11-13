# Phase 18.3.2: mutable borrow access safety

## Baseline and scope

The source baseline is `e00a1dc` (Phase 18.2), with 53 library tests and
6 CLI tests passing before changes. Formatting and Clippy also passed.
Work is on `phase18-3-2-borrow-safety`; existing untracked files and backups
are preserved. This phase changes the checker and adds regressions. It does
not implement general non-lexical lifetimes or change reference representation.

## Supported lifetime rule

An active mutable borrow prevents direct reads, assignment, moves, and new
borrows of its referent. Access through the exclusive reference remains valid.
Borrow creation checks the source binding's existence, moved state, type,
mutability, and conflicting loans; it is not an ordinary value read.

An exclusive borrow may end after a statement only when all of these hold:

1. The reference is declared in the current block.
2. The block contains only declarations, assignments, dereference assignments,
   and printing; there are no branches, loops, or exits in that block.
3. The block is outside every loop.
4. The statement mentions the existing reference and no later statement in
   that block mentions its name, including assignment or shadowing.
5. The reference binding still has the same identity after the statement.

The checker ends the loan after checking the entire statement, so a referent
read in the same expression as the last reference use still conflicts.
Unused references retain their loans until scope exit. Shared references
retain lexical lifetimes.

These restrictions make the local rule a single-path lifetime proof:
there is no back edge or exit inside the block, the reference has no subsequent
use, and no use can occur outside its declaring scope. Mutable references cannot
be copied, rebound, nested, or assigned into another reference binding under the
existing rules. Thus an accepted reference cannot escape this proof through an
alias. Repeated names are treated conservatively rather than resolving later
shadowed bindings as evidence of an earlier death.

This deliberately does not infer reference death across control flow. A reference
declared outside a branch remains live inside it and after the join. Branch-local
references disappear at scope exit. References inside loops retain lexical loans,
including references recreated in each iteration.

## Conservative control-flow restrictions

- Moving a String declared outside the innermost loop into another binding in
  that loop is rejected. Loop-local String moves remain supported.
- Reassigning a reference declared outside the innermost loop is rejected.
- A binding moved on loop entry remains moved after the loop, even if an
  assignment in the body appears to initialize it. This prevents assignments
  after `break` or `continue` from restoring ownership in the checker.
- Shadowing a borrowed target is rejected in inner scopes as well as the same
  scope. The native backend currently resolves mutable-reference targets by
  name; permitting this shadowing could redirect writes to another binding.
- Branch-dependent reference origins remain rejected by the existing checker.

Some safe programs are rejected, including a direct read after the apparent
final use of an outer reference in a branch, a one-iteration outer String move,
or reinitialization of an already moved String exclusively inside a loop.
These restrictions need control-flow analysis before they can be relaxed.
The existing two-iteration loop check is retained as an additional check, not
claimed to be a fixed-point ownership analysis.

## Validation

Seven of the initial eight regression tests failed against the baseline; all
pass after the change. The complete suite now has 71 passing tests:
53 library tests, 6 CLI tests, and 12 borrow safety integration tests.

The Windows differential integration test checks eight programs against explicit
expected output in both the interpreter and linked native executables. Cases
cover repeated borrows, branch-local references, loop-local references, shared
reference reassignment outside loops, and mutable i32, f64, bool, and String
values. Child processes have a 20-second deadline. Native coverage requires
Windows and MSVC; other platforms skip that test.

All eight existing strict reference fixtures also passed with the rebuilt debug
compiler, including native runs for the three valid fixtures. Formatting, Clippy,
and `git diff --check` passed. The first PowerShell runner invocation was blocked
by local script execution policy; rerunning with a process-local execution policy
setting passed without changing persistent system policy.

Commands from the repository root:

```text
cargo test --locked --all-targets --manifest-path onion-compiler/Cargo.toml
cargo fmt --all --manifest-path onion-compiler/Cargo.toml -- --check
cargo clippy --locked --all-targets --manifest-path onion-compiler/Cargo.toml
```

## Remaining security work

General control-flow lifetime and ownership analysis remains unimplemented.
Parser/AST depth limits, unchecked native String allocation, output-path and
temporary-file protections, dependency advisory review, and fuzzing infrastructure
remain outside this phase. The existing PowerShell strict runner also needs
asynchronous stream handling before its timeout can be relied on for adversarial
processes. The new native differential test uses its own bounded process runner.

Passing these tests does not establish full language soundness or production
readiness. The next lifetime phase should define binding-resolved control flow,
backward liveness, loan propagation, and fixed-point joins for loops and exits
before expanding borrow shortening.
