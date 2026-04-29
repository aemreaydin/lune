# Lune Milestones

Milestones are learning gates, not rigid release dates. Each topic follows the
workflow in `ARCHITECTURE.md`: explanation, alternatives, industry examples,
architecture impact, failing tests, user implementation, review, cleanup.

Detailed submilestones and user stories live in
[`docs/submilestones.md`](submilestones.md). This file stays as the high-level
roadmap.

## Milestone 0: Repository Foundation

Goals:

- create Rust workspace
- add `rust-toolchain.toml` pinned to current stable Rust
- add root `Cargo.toml`
- add `justfile`
- add license files for `MIT OR Apache-2.0`
- add basic docs structure
- document development commands in [`development.md`](development.md)
- track asset licenses in [`../assets/LICENSES.md`](../assets/LICENSES.md)

Verification:

```bash
just fmt
just clippy
just test
```

## Milestone 1: Diagnostics and Config

Goals:

- `lune_diagnostics`
- typed error conventions with `thiserror`
- `tracing` subscriber setup
- `tracing-log` bridge if useful
- TOML config structs
- config merge: defaults + root `Lune.toml` + showcase `Showcase.toml`

Tools:

- `cargo fmt`
- `cargo clippy`
- `cargo nextest`
- `bacon`
- `cargo tree -d`

## Milestone 2: Memory and Collections

Goals:

- `lune_memory`
- simple allocator interfaces
- linear allocator
- frame allocator wrapper
- pool allocator
- generational handle allocator helpers
- allocation stats
- `lune_collections`
- `SmallVec<T, N>`
- `SmallString<N>`
- `FixedVec<T, N>`
- ring buffer

Tools:

- `cargo miri`
- `proptest`
- `criterion`

## Milestone 3: Entities and ECS Internals

Goals:

- `lune_core`
- generational `Entity`
- entity slot table
- archetype tables
- component columns
- add/remove/move components
- low-level query iteration
- static typed components first
- resources
- typed event queues

Tools:

- `cargo miri` for safe internal invariants where applicable
- `proptest` for entity/component lifecycle invariants
- `criterion` for spawn/despawn/query iteration

## Milestone 4: App, Schedule, Time, and Transforms

Goals:

- app builder
- compile-time plugin system
- explicit `System` trait
- deterministic stages
- variable update and fixed update support
- `lune_math`
- transform types
- ECS hierarchy components
- full transform propagation
- camera math

## Milestone 5: Platform and Input

Goals:

- `lune_platform`
- `winit` window/event loop
- `lune_input`
- keyboard/mouse abstraction
- input state resource
- basic action mapping

## Milestone 6: Asset Handles and Source Loading

Goals:

- `lune_asset`
- per-type generational `Handle<T>`
- asset registry
- loader trait
- synchronous load path
- PNG/JPEG image loading
- glTF source loading preparation

Tools:

- `cargo-deny`
- `cargo-machete`

## Milestone 7: Render API and Vulkan Bring-Up

Goals:

- `lune_render`
- renderer API and resource descriptions
- capability model
- `lune_render_vulkan`
- Vulkan instance/device/swapchain
- validation/debug utils integration
- ordered shutdown
- simple GPU allocator
- clear-color smoke mode

Tools:

- RenderDoc/manual GPU debugging
- `tracing`

## Milestone 8: Shaders, Triangle, and Textured Geometry

Goals:

- Slang compile path
- SPIR-V cache
- reflection metadata
- classic descriptor sets first
- sync2/timeline plus fallback paths
- triangle smoke mode
- textured cube smoke mode

Tools:

- `cargo insta` for shader reflection snapshots

## Milestone 9: Forward Renderer and PBR-Lite

Goals:

- forward render path
- interleaved vertex layout
- base color and normal textures
- metallic/roughness factors
- directional light and ambient
- camera uniform
- material/resource binding

## Milestone 10: Scene and glTF Static Showcase

Goals:

- `lune_scene`
- `lune_scene_gltf`
- glTF nodes to ECS entities
- transforms and hierarchy
- meshes, materials, textures
- free/debug camera and scene camera support
- `showcase_01_static_scene`

Done when:

- a real glTF static scene renders through ECS, assets, renderer extract, and
  Vulkan backend
- no temporary renderer bring-up code has leaked into permanent engine APIs

## Milestone 11: Hot Reload

Goals:

- file watching with `notify`
- asset reload queue
- shader reload
- texture/material reload
- script reload foundations
- keep old good asset on failure
- debug reload logs

## Milestone 12: Scripting

Goals:

- `lune_script`
- `lune_script_lua`
- safe gameplay facade
- command API
- isolated Lua environments
- script lifecycle callbacks
- simple scripted showcase

## Milestone 13: Physics, Animation, Audio

Goals:

- `lune_physics` facade
- `lune_physics_rapier`
- `lune_animation` custom runtime
- `lune_audio` facade
- `lune_audio_kira`

Physics and audio are lower learning priority than ECS/render/memory.
Animation is a deeper custom learning topic.

## Milestone 14: Jobs and Performance

Goals:

- `lune_task`
- thread pool/job system
- parallel-friendly scheduler design
- asset background decode/import
- animation jobs
- culling/batching preparation

Tools:

- `loom`
- `cargo flamegraph`
- `cargo bloat`
- `iai-callgrind`

## Milestone 15: Cooker and Advanced Tests

Goals:

- `lune_cooker`
- shader preloading
- offline asset cooking
- mesh layout optimization
- texture mip/compression pipeline
- asset dependency graph
- cooked binary format
- snapshot/golden tests
- fuzz parsers/importers

Tools:

- `cargo fuzz`
- `cargo-mutants`
- `cargo-semver-checks` later if public API stability matters
