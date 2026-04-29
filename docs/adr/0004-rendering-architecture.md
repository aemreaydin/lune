# ADR-0004: Rendering Architecture

## Status

Accepted

## Context

Lune should teach modern Vulkan and game renderer architecture without letting
Vulkan-specific types leak into the rest of the engine.

The renderer should prefer Vulkan 1.4-era functionality where available, but
must support fallback paths so the engine can run and be tested on more common
hardware.

## Decision

`lune_render` defines the renderer-facing API, resource descriptions,
capability model, and future render graph concepts. `lune_render_vulkan`
implements the Vulkan backend using `ash`.

No Vulkan types may be exposed outside `lune_render_vulkan`.

The Vulkan backend uses Vulkan 1.2 as the baseline, with modern paths for
features such as dynamic rendering, synchronization2, timeline semaphores,
shader objects, descriptor indexing, and related extensions as they are added.
Each modern feature must land with its fallback implementation at the same
time. Config/runtime settings must allow forcing modern or fallback paths when
supported, so both paths can be tested.

The first renderer path is forward rendering with PBR-lite materials. Initial
vertices contain position, normal, tangent, and UV0. Meshes use an interleaved
layout first, while the render API remains flexible enough for separate
attribute streams later.

Shader authoring starts with Slang. The development pipeline compiles Slang to
SPIR-V, extracts reflection metadata, validates resource layouts, and caches
compiled outputs. A fuller offline cooker comes later.

Vulkan resource management uses RAII wrappers plus explicit ordered shutdown.
The backend supports validation layers, debug object names, command labels, and
shader printf when supported. Validation and debug messages are routed through
`tracing`.

## Consequences

This creates more backend code than a single hardcoded Vulkan path, but keeps
the architecture honest and teaches both modern Vulkan features and fallback
engineering.
