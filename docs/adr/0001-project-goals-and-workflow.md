# ADR-0001: Project Goals and Learning Workflow

## Status

Accepted

## Context

Lune is a learning project. The goal is not only to produce a working engine,
but to understand why engine systems are designed the way they are in industry.
Core topics such as ECS, allocators, renderer architecture, asset handles, and
Vulkan resource management must be implemented by the user.

The AI is part of the workflow, but should not replace the learning work.

## Decision

For each topic, the AI will:

1. Explain the topic in detail.
2. Compare alternatives with pros and cons.
3. Give industry examples where useful.
4. Explain how the topic fits Lune architecture.
5. Write failing tests and implementation handles.
6. Review failures and help debug after the user implementation.

The AI may write docs, tests, crate skeletons, public API stubs, and simple
non-learning glue. The AI must not implement production logic for learning
topics unless explicitly asked.

Unit tests are the primary feedback loop. Integration, snapshot, golden, and
performance tests are added at milestone boundaries.

## Consequences

This keeps the project slower than a normal AI-assisted implementation, but
preserves the main value: learning system design by building the system.

Docs are treated as part of the implementation. The architecture decision
ledger in `ARCHITECTURE.md` is the current truth; ADRs explain major decisions.
