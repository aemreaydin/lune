# Lune

Lune is a learning-first 3D game engine written in Rust. The goal is to
understand industry-standard engine architecture by building the core systems
directly, with documentation, tests, and reviews guiding each milestone.

The project is intentionally modular. Current work focuses on foundational
crates such as diagnostics, configuration, memory allocators, collections, and
math before moving into ECS, platform, rendering, assets, and showcases.

## Repository

- [`ARCHITECTURE.md`](ARCHITECTURE.md): project direction, workflow rules, and
  technical decisions.
- [`docs/milestones.md`](docs/milestones.md): high-level roadmap.
- [`docs/submilestones.md`](docs/submilestones.md): detailed task and user-story
  breakdown.
- [`docs/development.md`](docs/development.md): local toolchain and command
  reference.
- [`docs/learning/`](docs/learning/): topic notes used during the learning
  workflow.

## Development

Lune pins Rust in [`rust-toolchain.toml`](rust-toolchain.toml). The common local
commands are:

```bash
just fmt
just clippy
just test
just doc-test
just check
```

`just check` is the standard pre-commit gate. It runs formatting checks, clippy,
doctests, and the workspace test suite through nextest.

## License

Lune is licensed under either of:

- Apache License, Version 2.0
- MIT license
