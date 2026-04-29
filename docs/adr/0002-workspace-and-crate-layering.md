# ADR-0002: Workspace and Crate Layering

## Status

Accepted

## Context

Lune should be library-heavy and modular. The engine needs clear subsystem
boundaries so rendering, assets, scripting, physics, audio, debug tools, and
showcases can evolve independently.

## Decision

Lune uses a single Rust workspace with focused `lune_*` crates.

Base crates such as `lune_memory`, `lune_collections`, `lune_math`, and
`lune_diagnostics` sit at the bottom. Runtime crates such as `lune_core`,
`lune_asset`, and `lune_render` build on those. Backend crates such as
`lune_render_vulkan`, `lune_script_lua`, `lune_physics_rapier`, and
`lune_audio_kira` implement facade APIs without leaking backend types upward.

The workspace starts with no Cargo feature matrix. Optionality is handled by
crate boundaries and runtime/config settings first.

## Consequences

This creates more crates than a monolith, but forces clean dependencies and
keeps Vulkan, Lua, Rapier, and Kira from shaping the entire engine.

Showcases live in the same workspace so they can track the current engine API.
