# ADR-0003: Core Learning Implementations

## Status

Accepted

## Context

Some engine systems are the reason the project exists. Replacing them with
third-party crates would reduce learning value. At the same time, the project
should not waste effort reimplementing commodity platform code.

## Decision

Lune custom-builds the following core learning areas:

- ECS, starting with archetype storage
- entity and asset generational handle patterns
- memory allocators and arenas
- game-oriented collections such as `SmallVec`, `SmallString`, `FixedVec`,
  ring buffers, and slot/generational storage helpers
- renderer architecture and Vulkan backend
- asset registry and hot reload architecture
- animation runtime
- job/task system

Third-party crates may be used where they do not replace the learning target.
For example, `ash`, `winit`, `glam`, `gltf`, `image`, `mlua`, `rapier3d`, and
`kira` are acceptable because they support engine work without replacing the
chosen core learning systems.

`bytemuck` and `zerocopy` may be used after an explicit memory-layout/POD
safety lesson. Crates such as `smallvec`, `arrayvec`, `slotmap`, `hecs`,
`bevy_ecs`, and GPU allocator crates are references, not dependencies for the
custom implementations.

Unsafe Rust is allowed only in low-level crates such as `lune_memory`,
`lune_collections`, `lune_render_vulkan`, and FFI/platform boundaries. Unsafe
blocks require safety comments. High-level crates and showcases must not use
unsafe.

## Consequences

The engine will take longer to build, but core systems will be understood at
the implementation level. Tests, benchmarks, Miri, and property tests are
important safeguards for these custom systems.
