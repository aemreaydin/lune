# Development Commands

## Toolchain

Lune pins Rust 1.95.0 in `rust-toolchain.toml`.

Expected local versions:

```bash
rustc 1.95.0
cargo 1.95.0
```

Rust 1.95.0 is the accepted stable baseline for M0.

## Required Tools

- `just`
- `cargo fmt`
- `cargo clippy`
- `cargo nextest`

Install `cargo-nextest` if `just test` reports it missing:

```bash
cargo install cargo-nextest --locked
```

The `just` recipes fail loudly when a required optional Cargo subcommand is
missing. They do not silently fall back to a weaker check.

## Commands

```bash
just fmt        # format the workspace
just fmt-check  # verify formatting
just clippy     # run clippy over all workspace targets
just test       # run workspace tests with cargo-nextest
just doc-test   # run doctests
just check      # fmt-check, clippy, nextest, doctests
just tree-dups  # report duplicate dependency versions
```

Later milestones use the extra recipes in `justfile` as their tools become
part of the active workflow.

`just test` passes with a warning when no tests exist yet. That keeps the M0
empty crate skeleton verifiable while still running every nextest-compatible
test as soon as learning milestones add them.
