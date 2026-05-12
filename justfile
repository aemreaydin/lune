set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

require-nextest:
    @command -v cargo-nextest >/dev/null || { echo "cargo-nextest is required. Install it with: cargo install cargo-nextest --locked"; exit 127; }

test: require-nextest
    cargo nextest run --workspace --no-tests warn

doc-test:
    cargo test --workspace --doc

check: fmt-check clippy doc-test test

tree-dups:
    cargo tree -d

deny:
    cargo deny check

machete:
    cargo machete

coverage:
    cargo llvm-cov nextest --workspace

miri:
    cargo +nightly miri test --workspace

bench:
    cargo bench --workspace

smoke mode="clear":
    cargo run -p lune_renderer_smoke -- --mode {{mode}}
