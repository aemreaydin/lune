# Lune Submilestones

This is the working task backlog for the milestone roadmap. Each task should be
small enough to explain, test, implement, and review without carrying a whole
subsystem in one step.

Task type:

- `Learning`: AI explains, writes failing tests/stubs, user implements.
- `Scaffold`: AI may create structure or glue that is not a core learning
  implementation.
- `Decision`: requires discussion before tests or implementation.

Each task preserves the rule from `ARCHITECTURE.md`: AI does not implement
production logic for learning topics unless explicitly asked.

## Milestone 0: Repository Foundation

### M0.1 Create Rust Workspace Skeleton

Type: Scaffold

Blocked by: none

User story: As a Lune developer, I want a Rust workspace with the planned crate
folders so future topics have stable places to land.

Acceptance criteria:

- [x] root `Cargo.toml` exists with workspace members
- [x] crate directories exist for the initial foundation crates
- [x] workspace uses Rust edition 2024
- [x] workspace package metadata includes `rust-version`

### M0.2 Pin Toolchain

Type: Scaffold

Blocked by: M0.1

User story: As a Lune developer, I want a pinned stable Rust toolchain so all
examples and tests use the same compiler.

Acceptance criteria:

- [x] `rust-toolchain.toml` exists
- [x] toolchain matches the current accepted stable version
- [x] `cargo --version` matches the documented expectation

### M0.3 Add License Files

Type: Scaffold

Blocked by: none

User story: As a future library consumer, I want Lune's license policy explicit.

Acceptance criteria:

- [x] `LICENSE-MIT` exists
- [x] `LICENSE-APACHE` exists
- [x] root metadata declares `MIT OR Apache-2.0`
- [x] asset license tracking location is documented

### M0.4 Add Initial Quality Commands

Type: Scaffold

Blocked by: M0.1

User story: As a Lune developer, I want one command that runs formatting,
linting, tests, and doctests.

Acceptance criteria:

- [x] `just check` runs fmt check, clippy, nextest, and doctests
- [x] missing optional tools are documented, not silently ignored
- [x] commands are listed in docs

### M0.5 Add Learning Topic Template

Type: Scaffold

Blocked by: none

User story: As a learner, I want every implementation topic to follow the same
explain-first structure.

Acceptance criteria:

- [x] `docs/learning/TEMPLATE.md` exists
- [x] template includes alternatives, industry examples, Lune impact, tests,
  and implementation handles
- [x] `ARCHITECTURE.md` links the workflow

## Milestone 1: Diagnostics and Config

### M1.1 Explain Error and Logging Policy

Type: Learning

Blocked by: M0.5

User story: As a Lune developer, I want to understand why libraries use typed
errors and why engine code uses structured tracing.

Acceptance criteria:

- [x] learning doc explains `thiserror`, `anyhow`, `tracing`, and `log`
- [x] alternatives and tradeoffs are documented
- [x] failing tests define the expected diagnostics/config API shape

### M1.2 Create `lune_diagnostics` API Stubs

Type: Scaffold

Blocked by: M1.1

User story: As a crate author, I want common diagnostics types and setup
functions available without duplicating logging code.

Acceptance criteria:

- [x] `lune_diagnostics` exists
- [x] public setup API is stubbed
- [x] typed error examples compile
- [x] tests fail at intentional `todo!()` boundaries

### M1.3 Implement Tracing Subscriber Setup

Type: Learning

Blocked by: M1.2

User story: As a developer running a showcase, I want structured logs with
subsystem context and configurable verbosity.

Acceptance criteria:

- [x] subscriber setup can be called once safely
- [x] log filtering can be configured
- [x] dependency `log` records can be bridged into `tracing` if enabled
- [x] tests verify init behavior where practical

### M1.4 Define Config Types and Merge Semantics

Type: Learning

Blocked by: M1.2

User story: As a showcase author, I want root config defaults and showcase
overrides to produce one effective config.

Acceptance criteria:

- [x] learning doc explains config layering
- [x] config structs deserialize from TOML
- [x] defaults + root + showcase merge is tested
- [x] nested tables merge and arrays replace
- [x] unknown fields error in strict mode

## Milestone 2: Memory and Collections

### M2.1 Explain Engine Memory Patterns

Type: Learning

Blocked by: M0.5

User story: As a learner, I want to understand arenas, frame allocators, pools,
alignment, lifetimes, and why engines avoid hot-path heap churn.

Acceptance criteria:

- [x] learning doc covers allocator alternatives and engine examples
- [x] unsafe boundaries and invariants are documented
- [x] tests define expected allocator behavior before implementation

### M2.2 Create `lune_memory` Allocator Interfaces

Type: Learning

Blocked by: M2.1

User story: As a subsystem author, I want a small allocator API to use in
engine-owned memory paths.

Acceptance criteria:

- [ ] allocator traits/types are defined
- [ ] allocation result carries pointer/size/alignment metadata as needed
- [ ] failure behavior is explicit
- [ ] tests compile and fail until implementation exists

### M2.3 Implement Linear Allocator

Type: Learning

Blocked by: M2.2

User story: As a renderer/ECS implementer, I want a simple bump allocator for
linear allocation patterns.

Acceptance criteria:

- [ ] aligned allocations succeed when capacity allows
- [ ] out-of-memory returns typed error
- [ ] reset reuses memory
- [ ] Miri-relevant tests pass where applicable
- [ ] allocation stats are updated

### M2.4 Implement Frame Allocator Wrapper

Type: Learning

Blocked by: M2.3

User story: As a renderer developer, I want transient per-frame allocations that
can be reset at frame boundaries.

Acceptance criteria:

- [ ] frame reset invalidation rules are documented
- [ ] allocation and reset behavior is tested
- [ ] misuse cases are covered by API design or tests

### M2.5 Implement Pool Allocator

Type: Learning

Blocked by: M2.2

User story: As a system author, I want fixed-size object allocation with stable
reuse behavior.

Acceptance criteria:

- [ ] allocate/free/reuse behavior is tested
- [ ] double-free is detected or prevented
- [ ] capacity and alignment behavior is tested
- [ ] stats expose active and free slots

### M2.6 Explain Small Game Containers

Type: Learning

Blocked by: M0.5

User story: As a learner, I want to understand inline storage containers and why
engines use small vector/string types.

Acceptance criteria:

- [ ] learning doc covers `SmallVec`, `SmallString`, `FixedVec`, and ring buffers
- [ ] reference crates are discussed but not depended on
- [ ] test plan emphasizes boundary conditions

### M2.7 Implement `FixedVec<T, N>`

Type: Learning

Blocked by: M2.6

User story: As a systems programmer, I want a no-heap vector for fixed-capacity
hot-path data.

Acceptance criteria:

- [ ] push/pop/clear/iteration behavior is tested
- [ ] capacity overflow is explicit
- [ ] drop behavior is tested
- [ ] Miri passes for this crate

### M2.8 Implement `SmallVec<T, N>`

Type: Learning

Blocked by: M2.7

User story: As a systems programmer, I want inline storage with heap spillover
for common small collections.

Acceptance criteria:

- [ ] inline path is tested
- [ ] spill path is tested
- [ ] moves, drops, and capacity transitions are tested
- [ ] property tests cover push/pop equivalence with `Vec`

### M2.9 Implement `SmallString<N>`

Type: Learning

Blocked by: M2.7

User story: As a system author, I want small UTF-8 strings without heap
allocation for common names and labels.

Acceptance criteria:

- [ ] valid UTF-8 invariant is preserved
- [ ] inline and spill paths are tested
- [ ] push/clear/formatting behavior is tested
- [ ] invalid boundary cases are tested

### M2.10 Implement Ring Buffer

Type: Learning

Blocked by: M2.7

User story: As an engine developer, I want bounded FIFO storage for events,
logs, and future job queues.

Acceptance criteria:

- [ ] wraparound behavior is tested
- [ ] full/empty behavior is explicit
- [ ] iteration order is tested
- [ ] overwrite vs reject policy is documented

## Milestone 3: Entities and ECS Internals

### M3.1 Explain Generational Handles

Type: Learning

Blocked by: M2.2

User story: As a learner, I want to understand stale handle prevention for
entities, assets, and resources.

Acceptance criteria:

- [ ] learning doc covers index/generation/free-list pattern
- [ ] tests define spawn/despawn/reuse behavior
- [ ] overflow and invalid handle policy is documented

### M3.2 Implement Entity Slot Table

Type: Learning

Blocked by: M3.1

User story: As an ECS implementer, I want entity IDs that validate against a
slot table.

Acceptance criteria:

- [ ] spawn creates live entities
- [ ] despawn invalidates old handles
- [ ] recycled index increments generation
- [ ] invalid/despawned entities are rejected

### M3.3 Explain Archetype ECS Storage

Type: Learning

Blocked by: M3.2

User story: As a learner, I want to understand archetypes, component columns,
structural moves, and query iteration performance.

Acceptance criteria:

- [ ] learning doc compares archetype vs sparse-set vs hybrid ECS
- [ ] industry examples are included
- [ ] tests define archetype/table invariants

### M3.4 Implement Component Type Registry

Type: Learning

Blocked by: M3.3

User story: As an ECS implementer, I want stable component type metadata for
archetype layout and queries.

Acceptance criteria:

- [ ] component type IDs are stable within a run
- [ ] metadata includes size/alignment/type name
- [ ] duplicate registration behavior is tested

### M3.5 Implement Archetype Tables

Type: Learning

Blocked by: M3.4

User story: As an ECS implementer, I want rows of entities with columnar
component storage.

Acceptance criteria:

- [ ] entities insert into matching archetypes
- [ ] component columns preserve row alignment
- [ ] row swap-remove updates entity locations
- [ ] tests cover empty and multi-component archetypes

### M3.6 Implement Component Add/Remove Moves

Type: Learning

Blocked by: M3.5

User story: As a gameplay system author, I want adding/removing components to
move entities between archetypes without changing entity handles.

Acceptance criteria:

- [ ] add component moves entity to new archetype
- [ ] remove component moves entity to new archetype
- [ ] unchanged components are preserved
- [ ] entity location table updates correctly

### M3.7 Implement Low-Level Query Iteration

Type: Learning

Blocked by: M3.5

User story: As a system author, I want to iterate all entities matching a set of
component types.

Acceptance criteria:

- [ ] queries find matching archetypes
- [ ] non-matching archetypes are skipped
- [ ] immutable and mutable access rules are documented
- [ ] iteration order is deterministic within documented limits

### M3.8 Add Minimal Resources

Type: Learning

Blocked by: M3.2

User story: As an engine system, I want typed singleton resources such as time,
input, and asset registries.

Acceptance criteria:

- [ ] insert/get/get_mut/remove behavior is tested
- [ ] missing resource errors are explicit
- [ ] duplicate insert policy is tested

### M3.9 Add Typed Event Queues

Type: Learning

Blocked by: M3.8

User story: As a system author, I want typed events that systems can write and
read per frame.

Acceptance criteria:

- [ ] events can be sent and read
- [ ] clear/end-frame behavior is tested
- [ ] double-buffer semantics are documented if used

### M3.10 Add ECS Benchmarks

Type: Learning

Blocked by: M3.7

User story: As an engine developer, I want initial performance baselines for
entity and query operations.

Acceptance criteria:

- [ ] criterion benchmark for spawn/despawn
- [ ] criterion benchmark for query iteration
- [ ] results are documented as baseline, not optimization target

## Milestone 4: App, Schedule, Time, and Transforms

### M4.1 Explain Engine App and Plugin Model

Type: Learning

Blocked by: M3.8

User story: As a learner, I want to understand app builders, plugins, stages,
and why engines centralize system registration.

Acceptance criteria:

- [ ] learning doc compares direct setup vs plugins
- [ ] tests define plugin registration behavior
- [ ] compile-time plugin boundary is documented

### M4.2 Implement App Builder and Plugin Trait

Type: Learning

Blocked by: M4.1

User story: As a showcase author, I want to assemble engine subsystems through a
readable app builder.

Acceptance criteria:

- [ ] plugins can insert resources and systems
- [ ] plugin order is deterministic
- [ ] duplicate plugin policy is documented
- [ ] showcase-like test demonstrates setup

### M4.3 Implement Simple Scheduler

Type: Learning

Blocked by: M4.2

User story: As an engine developer, I want systems to run in deterministic
stages.

Acceptance criteria:

- [ ] stages exist for startup/update/fixed/update/render extraction/render
- [ ] systems run in insertion order within a stage
- [ ] failing systems/errors are surfaced
- [ ] tests verify order

### M4.4 Implement Time and Fixed-Step Accumulator

Type: Learning

Blocked by: M4.3

User story: As a simulation system, I want variable update timing and fixed
physics timing.

Acceptance criteria:

- [ ] variable `dt` is tracked
- [ ] fixed-step accumulator runs correct number of steps
- [ ] max fixed steps prevents spiral behavior
- [ ] tests cover large frame times

### M4.5 Explain Transform Conventions

Type: Learning

Blocked by: M3.7

User story: As a learner, I want to understand coordinate systems, local/global
transforms, matrix layout, and Vulkan/glTF implications.

Acceptance criteria:

- [ ] learning doc states right-handed/Y-up/forward `-Z`
- [ ] Vulkan clip-space concerns are documented
- [ ] transform tests are specified

### M4.6 Implement Transform Components

Type: Learning

Blocked by: M4.5

User story: As a scene system, I want local and global transforms represented as
ECS components.

Acceptance criteria:

- [ ] local transform to matrix is tested
- [ ] global transform composition is tested
- [ ] units/radians conventions are documented in API docs

### M4.7 Implement Hierarchy Components and Propagation

Type: Learning

Blocked by: M4.6

User story: As a glTF importer, I want parent/child transforms to compute
correct global transforms.

Acceptance criteria:

- [ ] root and child propagation is tested
- [ ] multi-level hierarchy is tested
- [ ] reparent/remove behavior is defined or explicitly deferred

### M4.8 Implement Camera Math

Type: Learning

Blocked by: M4.6

User story: As a renderer, I want camera view/projection matrices consistent
with Lune's world conventions.

Acceptance criteria:

- [ ] perspective camera matrix is tested
- [ ] view matrix from transform is tested
- [ ] Vulkan projection adjustments are tested

## Milestone 5: Platform and Input

### M5.1 Explain Window/Event Loop Architecture

Type: Learning

Blocked by: M4.2

User story: As a learner, I want to understand why platform events are hidden
behind Lune abstractions.

Acceptance criteria:

- [ ] learning doc compares raw `winit` exposure vs abstraction
- [ ] platform ownership model is documented
- [ ] tests/stubs define expected platform API

### M5.2 Create `lune_platform` Winit Shell

Type: Scaffold

Blocked by: M5.1

User story: As a showcase, I want a window and event loop without directly
depending on `winit` types outside the platform crate.

Acceptance criteria:

- [ ] platform crate owns `winit` integration
- [ ] window config comes from effective config
- [ ] platform events convert to Lune event types

### M5.3 Explain Input Abstraction

Type: Learning

Blocked by: M5.2

User story: As a learner, I want to understand input state, action maps, raw
events, and future replay/testing concerns.

Acceptance criteria:

- [ ] learning doc covers keyboard/mouse and future gamepad support
- [ ] tests define input state transitions
- [ ] action mapping policy is documented

### M5.4 Implement Keyboard and Mouse State

Type: Learning

Blocked by: M5.3

User story: As a camera controller, I want current and edge-triggered input
state.

Acceptance criteria:

- [ ] pressed/released/held states are tested
- [ ] mouse delta and cursor state are tested
- [ ] frame boundary reset behavior is tested

### M5.5 Implement Basic Action Map

Type: Learning

Blocked by: M5.4

User story: As a gameplay system, I want semantic actions instead of raw key
checks.

Acceptance criteria:

- [ ] config can bind actions to keys/buttons
- [ ] action pressed/held/released is tested
- [ ] missing/duplicate binding behavior is documented

## Milestone 6: Asset Handles and Source Loading

### M6.1 Explain Asset Handles and Registry

Type: Learning

Blocked by: M3.1

User story: As a learner, I want to understand why runtime code uses typed
generational handles instead of paths.

Acceptance criteria:

- [ ] learning doc covers path strings, plain handles, generational handles,
  and global asset IDs
- [ ] tests define stale handle behavior
- [ ] hot-reload implications are documented

### M6.2 Implement Typed `Handle<T>`

Type: Learning

Blocked by: M6.1

User story: As a runtime system, I want type-safe asset references that detect
stale slots.

Acceptance criteria:

- [ ] handles are typed and compact
- [ ] stale handle validation is tested
- [ ] invalid/null handle policy is documented

### M6.3 Implement Asset Registry Storage

Type: Learning

Blocked by: M6.2

User story: As an asset system, I want to store assets by type and resolve
typed handles.

Acceptance criteria:

- [ ] insert/get/remove behavior is tested
- [ ] stale lookup fails safely
- [ ] metadata includes source path where applicable

### M6.4 Implement Loader Trait and Sync Loading

Type: Learning

Blocked by: M6.3

User story: As a showcase, I want source files loaded synchronously with typed
errors.

Acceptance criteria:

- [ ] loader trait is generic over asset type
- [ ] load success registers asset
- [ ] load failure returns typed error
- [ ] async future path is documented, not implemented

### M6.5 Implement Image Loading Basics

Type: Learning

Blocked by: M6.4

User story: As a renderer, I want PNG/JPEG textures decoded with correct color
space metadata.

Acceptance criteria:

- [ ] image decode path creates texture asset data
- [ ] sRGB vs linear intent is represented
- [ ] errors include path/source context

### M6.6 Add Dependency Hygiene Tools

Type: Scaffold

Blocked by: M0.4

User story: As a maintainer, I want dependency license/advisory checks and
unused dependency checks available.

Acceptance criteria:

- [ ] `cargo-deny` config exists
- [ ] `cargo-machete` usage is documented
- [ ] commands are available through `just`

## Milestone 7: Render API and Vulkan Bring-Up

### M7.1 Explain RHI and Backend Boundaries

Type: Learning

Blocked by: M4.8

User story: As a learner, I want to understand why `lune_render` hides Vulkan
behind backend-neutral APIs.

Acceptance criteria:

- [ ] learning doc compares direct Vulkan exposure vs RHI-style API
- [ ] no-Vulkan-leak rule is documented
- [ ] tests/stubs define render resource descriptions

### M7.2 Define Render API Resource Descriptions

Type: Learning

Blocked by: M7.1

User story: As a renderer backend, I want backend-neutral descriptions for
buffers, textures, samplers, shaders, and pipelines.

Acceptance criteria:

- [ ] public render API contains no Vulkan types
- [ ] descriptors are documented
- [ ] tests verify type boundaries where practical

### M7.3 Explain Vulkan Instance, Device, Queues, and Swapchain

Type: Learning

Blocked by: M7.2

User story: As a learner, I want to understand Vulkan initialization and the
ownership/lifetime chain.

Acceptance criteria:

- [ ] learning doc covers instance/device/queues/surfaces/swapchain
- [ ] validation/debug setup is explained
- [ ] ordered shutdown plan is documented

### M7.4 Create Vulkan Backend Shell

Type: Learning

Blocked by: M7.3

User story: As a showcase, I want a Vulkan backend object that can initialize
and shut down cleanly.

Acceptance criteria:

- [ ] Vulkan backend is constructed through `lune_render` API
- [ ] validation messages route to `tracing`
- [ ] shutdown order is explicit
- [ ] raw handles remain private

### M7.5 Implement Capability Detection and Forcing

Type: Learning

Blocked by: M7.4

User story: As a renderer developer, I want to detect supported modern features
and force fallback paths for testing.

Acceptance criteria:

- [ ] capabilities are recorded in backend-neutral structs
- [ ] config can request auto/modern/fallback
- [ ] unsupported forced modes produce clear errors or warnings
- [ ] selection logic is unit tested without GPU

### M7.6 Explain Vulkan GPU Memory Basics

Type: Learning

Blocked by: M7.4

User story: As a learner, I want to understand heaps, memory types, allocation
requirements, dedicated allocations, and suballocation.

Acceptance criteria:

- [ ] learning doc covers Vulkan memory model
- [ ] simple allocator strategy is documented
- [ ] tests define memory-type selection logic

### M7.7 Implement Simple Vulkan GPU Allocator

Type: Learning

Blocked by: M7.6

User story: As a Vulkan backend, I want simple GPU allocations tracked through
Lune memory policy.

Acceptance criteria:

- [ ] memory type selection is tested
- [ ] dedicated allocation path exists
- [ ] stats are tracked
- [ ] debug names are attached where supported

### M7.8 Render Clear Color Smoke Mode

Type: Learning

Blocked by: M7.5, M7.7

User story: As a renderer developer, I want the smallest visual proof that
swapchain acquire, command submit, present, and shutdown work.

Acceptance criteria:

- [ ] `lune_renderer_smoke --mode clear` opens a window
- [ ] frame loop clears to configured color
- [ ] resize/minimize behavior is handled or explicitly deferred
- [ ] validation errors are absent under normal run

## Milestone 8: Shaders, Triangle, and Textured Geometry

### M8.1 Explain Slang, SPIR-V, and Reflection

Type: Learning

Blocked by: M7.2

User story: As a learner, I want to understand shader compilation, reflection,
resource layouts, and why Lune starts with Slang.

Acceptance criteria:

- [ ] learning doc compares GLSL/HLSL/WGSL/Slang
- [ ] reflection metadata needs are documented
- [ ] snapshot tests are planned

### M8.2 Implement Slang Compile Shell

Type: Learning

Blocked by: M8.1

User story: As a renderer developer, I want shader source compiled into SPIR-V
with clear errors.

Acceptance criteria:

- [ ] compile path produces SPIR-V bytes or typed error
- [ ] source path and entry point are reported on failure
- [ ] cache output location is documented

### M8.3 Implement Shader Reflection Metadata

Type: Learning

Blocked by: M8.2

User story: As a render API user, I want shader resource bindings validated
against reflected metadata.

Acceptance criteria:

- [ ] reflection includes entry points, bindings, push constants, and stages
- [ ] `insta` snapshots cover reflection output
- [ ] mismatched pipeline layout produces typed error

### M8.4 Explain Vulkan Sync Modern and Fallback Paths

Type: Learning

Blocked by: M7.4

User story: As a learner, I want to understand timeline semaphores,
binary semaphores, fences, sync2, and legacy barriers.

Acceptance criteria:

- [ ] learning doc covers modern and fallback sync
- [ ] config forcing model is documented
- [ ] tests cover path-selection logic

### M8.5 Implement Sync Path Selection

Type: Learning

Blocked by: M8.4, M7.5

User story: As a backend, I want sync code paths selected by capability and
config.

Acceptance criteria:

- [ ] timeline vs binary mode selection is tested
- [ ] sync2 vs legacy barrier selection is tested
- [ ] unsupported forced choices fail clearly

### M8.6 Render Triangle Smoke Mode

Type: Learning

Blocked by: M8.3, M8.5

User story: As a renderer developer, I want a known triangle path to validate
shader modules, pipeline state, command recording, and presentation.

Acceptance criteria:

- [ ] triangle mode draws a visible triangle
- [ ] no temporary Vulkan types leak into `lune_render`
- [ ] shader errors are clear
- [ ] fallback/modern path can be selected where implemented

### M8.7 Render Textured Cube Smoke Mode

Type: Learning

Blocked by: M8.6, M6.5

User story: As a renderer developer, I want textured geometry to validate
buffers, images, samplers, descriptors, and depth.

Acceptance criteria:

- [ ] textured cube mode draws with depth
- [ ] texture upload path is exercised
- [ ] descriptor set layout is validated against reflection
- [ ] stale bring-up code is removed or kept only inside smoke mode

## Milestone 9: Forward Renderer and PBR-Lite

### M9.1 Explain Forward Rendering and PBR-Lite

Type: Learning

Blocked by: M8.7

User story: As a learner, I want to understand forward rendering tradeoffs and
why Lune starts with PBR-lite.

Acceptance criteria:

- [ ] learning doc compares unlit, forward, deferred, and forward+
- [ ] PBR-lite limitations are documented
- [ ] tests define CPU-side material/light data behavior

### M9.2 Define Mesh and Vertex Layout Types

Type: Learning

Blocked by: M9.1

User story: As an asset importer and renderer, I want mesh data with position,
normal, tangent, and UV0 represented clearly.

Acceptance criteria:

- [ ] interleaved layout is represented
- [ ] future separate-stream support is not blocked
- [ ] layout/stride/attribute tests exist

### M9.3 Define Material Asset Types

Type: Learning

Blocked by: M9.1, M6.3

User story: As a renderer, I want fixed PBR-lite material inputs backed by
asset handles.

Acceptance criteria:

- [ ] material stores base color, normal, metallic, roughness data
- [ ] texture handles are typed
- [ ] defaults match documented behavior

### M9.4 Implement Render Extract Data

Type: Learning

Blocked by: M4.8, M9.2, M9.3

User story: As a renderer, I want gameplay ECS data converted into renderer-owned
draw data.

Acceptance criteria:

- [ ] extract gathers camera, transform, mesh, material, light data
- [ ] renderer does not borrow gameplay world during draw
- [ ] tests use fake renderer/extract data

### M9.5 Render PBR-Lite Forward Scene

Type: Learning

Blocked by: M9.4

User story: As a showcase, I want multiple textured meshes lit by a directional
light through the forward renderer.

Acceptance criteria:

- [ ] base color texture/factor is visible
- [ ] normal map path is represented
- [ ] metallic/roughness factors are accepted
- [ ] ambient + directional light are applied

## Milestone 10: Scene and glTF Static Showcase

### M10.1 Explain glTF Import and Runtime Scene Boundaries

Type: Learning

Blocked by: M9.2, M9.3

User story: As a learner, I want to understand glTF as interchange format and
why Lune still needs runtime scene data.

Acceptance criteria:

- [ ] learning doc maps glTF nodes/meshes/materials/textures to Lune concepts
- [ ] ignored features are documented
- [ ] importer tests are planned with fixtures

### M10.2 Implement glTF Mesh and Material Import

Type: Learning

Blocked by: M10.1, M6.4

User story: As a scene loader, I want glTF mesh primitives and materials turned
into Lune asset records.

Acceptance criteria:

- [ ] mesh attributes import into initial vertex format
- [ ] missing optional tangents policy is documented
- [ ] materials map to PBR-lite inputs
- [ ] importer errors include source context

### M10.3 Implement glTF Node to ECS Spawn

Type: Learning

Blocked by: M10.2, M4.7

User story: As a scene loader, I want glTF node hierarchy spawned into ECS
entities with transforms and render components.

Acceptance criteria:

- [ ] nodes create entities
- [ ] parent/child relationships are preserved
- [ ] local/global transforms match fixtures
- [ ] mesh/material handles attach to render components

### M10.4 Add Free Camera Controller

Type: Learning

Blocked by: M5.5, M4.8

User story: As a developer, I want to move through static scenes for inspection.

Acceptance criteria:

- [ ] camera movement uses input actions
- [ ] mouse look behavior is documented
- [ ] camera speed/config is testable where practical

### M10.5 Import and Select Scene Cameras

Type: Learning

Blocked by: M10.3, M4.8

User story: As a scene viewer, I want to use a glTF camera or override it with a
free camera.

Acceptance criteria:

- [ ] glTF cameras import into camera components/assets
- [ ] active camera can be selected by config
- [ ] fallback free camera exists if no scene camera is present

### M10.6 Ship `showcase_01_static_scene`

Type: Learning

Blocked by: M9.5, M10.3, M10.4, M10.5

User story: As a learner, I want the first complete engine slice: ECS, assets,
scene import, renderer extraction, and Vulkan draw working together.

Acceptance criteria:

- [ ] showcase loads configured glTF scene
- [ ] scene renders with PBR-lite forward renderer
- [ ] debug diagnostics show frame stats/logging basics
- [ ] temporary bring-up APIs are removed

## Milestone 11: Hot Reload

### M11.1 Explain Asset Hot Reload Architecture

Type: Learning

Blocked by: M10.6

User story: As a learner, I want to understand watchers, reload queues,
dependency tracking, and safe resource replacement.

Acceptance criteria:

- [ ] learning doc covers reload failure policy
- [ ] main-thread apply model is documented
- [ ] tests define reload state transitions

### M11.2 Add File Watch Event Source

Type: Learning

Blocked by: M11.1

User story: As an asset system, I want source file changes converted into typed
reload events.

Acceptance criteria:

- [ ] `notify` integration is isolated
- [ ] debouncing/coalescing policy is documented
- [ ] fake watcher tests cover event conversion

### M11.3 Implement Asset Reload Queue

Type: Learning

Blocked by: M11.2, M6.3

User story: As a running showcase, I want changed assets reloaded at safe points
without mutating resources mid-use.

Acceptance criteria:

- [ ] reload requests queue and apply on main thread
- [ ] success atomically swaps new asset data
- [ ] failure keeps old good asset
- [ ] diagnostics record success/failure

### M11.4 Implement Shader Reload

Type: Learning

Blocked by: M11.3, M8.3

User story: As a renderer developer, I want Slang shader edits to recompile and
update renderer resources.

Acceptance criteria:

- [ ] shader change triggers compile/reflection
- [ ] compatible shader swap succeeds
- [ ] incompatible change reports error and keeps old shader

### M11.5 Implement Texture/Material Reload

Type: Learning

Blocked by: M11.3, M9.3

User story: As an artist/developer, I want texture and material edits to appear
without restarting.

Acceptance criteria:

- [ ] texture reload updates GPU resource safely
- [ ] material reload updates CPU/GPU material data
- [ ] failed reload keeps old visible asset

## Milestone 12: Scripting

### M12.1 Explain Lua Scripting Boundary

Type: Learning

Blocked by: M4.2, M3.7

User story: As a learner, I want to understand why scripts use a facade and
commands instead of raw ECS mutation.

Acceptance criteria:

- [ ] learning doc compares Lua/Rhai/WASM/custom scripting
- [ ] facade and command API boundaries are documented
- [ ] tests define script lifecycle expectations

### M12.2 Create Script Facade and Command API

Type: Learning

Blocked by: M12.1

User story: As gameplay code, I want safe operations such as transform changes,
spawn, and despawn without raw ECS access.

Acceptance criteria:

- [ ] command API records operations
- [ ] command apply step mutates ECS safely
- [ ] invalid entity behavior is tested

### M12.3 Embed Lua VM and Isolated Environments

Type: Learning

Blocked by: M12.2

User story: As a script runner, I want one VM with isolated script environments
and stable bindings.

Acceptance criteria:

- [ ] scripts run in isolated env tables
- [ ] engine bindings expose only facade methods
- [ ] script errors include source context

### M12.4 Implement Script Lifecycle

Type: Learning

Blocked by: M12.3

User story: As a gameplay entity, I want `on_start`, `on_update`, and optional
`on_destroy` callbacks.

Acceptance criteria:

- [ ] lifecycle order is tested
- [ ] `dt` reaches update callback
- [ ] callback errors are surfaced without panicking engine

### M12.5 Implement Script Hot Reload

Type: Learning

Blocked by: M12.4, M11.3

User story: As a gameplay programmer, I want script edits to restart the script
instance while preserving entity/components.

Acceptance criteria:

- [ ] old `on_destroy` is called if present
- [ ] new script compiles and calls `on_start`
- [ ] failed reload keeps old script or disables according to documented policy

## Milestone 13: Physics, Animation, Audio

### M13.1 Add Physics Facade and Rapier Backend

Type: Learning

Blocked by: M4.4, M4.7

User story: As a gameplay system, I want fixed-step rigid body simulation
through a Lune physics API.

Acceptance criteria:

- [ ] facade hides Rapier types from core ECS
- [ ] fixed update drives physics step
- [ ] transform sync policy is documented

### M13.2 Explain Skeletal Animation Runtime

Type: Learning

Blocked by: M10.2

User story: As a learner, I want to understand clips, tracks, skeletons, poses,
interpolation, and skinning.

Acceptance criteria:

- [ ] learning doc maps glTF animation data to Lune runtime
- [ ] CPU skinning first and GPU skinning later are compared
- [ ] tests define interpolation and pose behavior

### M13.3 Implement Animation Clip Sampling

Type: Learning

Blocked by: M13.2

User story: As an animation system, I want to sample keyframe tracks into local
bone transforms.

Acceptance criteria:

- [ ] translation/rotation/scale interpolation is tested
- [ ] looping/clamping behavior is defined
- [ ] invalid track data errors are explicit

### M13.4 Implement Skeleton Pose and CPU Skinning

Type: Learning

Blocked by: M13.3

User story: As a renderer, I want animated meshes deformed by skeleton poses.

Acceptance criteria:

- [ ] joint hierarchy computes matrices
- [ ] inverse bind matrices are applied
- [ ] CPU skinning correctness is tested on tiny fixtures

### M13.5 Add Audio Facade and Kira Backend

Type: Learning

Blocked by: M4.2

User story: As a showcase, I want sound playback through a Lune audio API while
Kira owns low-level audio details.

Acceptance criteria:

- [ ] facade hides Kira types from gameplay
- [ ] sound asset handles are used
- [ ] play/stop/volume behavior is tested with fake backend where practical

## Milestone 14: Jobs and Performance

### M14.1 Explain Job Systems and Scheduler Parallelism

Type: Learning

Blocked by: M4.3

User story: As a learner, I want to understand task graphs, work stealing,
system dependencies, and data hazards.

Acceptance criteria:

- [ ] learning doc compares thread pool/job graph approaches
- [ ] ECS borrow/conflict implications are documented
- [ ] `loom` use cases are planned

### M14.2 Implement Basic Thread Pool

Type: Learning

Blocked by: M14.1

User story: As an engine subsystem, I want background work execution for asset
decode/import and future animation jobs.

Acceptance criteria:

- [ ] jobs execute and return results
- [ ] shutdown joins workers cleanly
- [ ] panic/error behavior is documented
- [ ] concurrency tests use `loom` where practical

### M14.3 Add Async Asset Decode/Import Path

Type: Learning

Blocked by: M14.2, M6.4

User story: As a runtime system, I want CPU-heavy asset work off the main thread
while GPU upload remains safe.

Acceptance criteria:

- [ ] asset state includes loading/ready/error
- [ ] completed CPU work queues main-thread upload
- [ ] sync load path still works

### M14.4 Add Performance Profiling Hooks

Type: Learning

Blocked by: M4.3

User story: As an optimizer, I want frame stats, scopes, and counters around hot
engine paths.

Acceptance criteria:

- [ ] `lune_perf` exposes scope/counter APIs
- [ ] tracing spans integrate with profiling
- [ ] frame stats are visible to debug tooling

### M14.5 Profile ECS and Renderer Hot Paths

Type: Learning

Blocked by: M10.6, M14.4

User story: As a learner, I want to use profiling tools to find real bottlenecks
instead of guessing.

Acceptance criteria:

- [ ] `cargo flamegraph` workflow is documented
- [ ] `cargo bloat` workflow is documented
- [ ] benchmark results are compared against earlier baselines

## Milestone 15: Cooker and Advanced Tests

### M15.1 Explain Offline Asset Cooking

Type: Learning

Blocked by: M10.6

User story: As a learner, I want to understand source assets, cooked assets,
dependency graphs, caches, and runtime preload.

Acceptance criteria:

- [ ] learning doc compares runtime import vs offline cooking
- [ ] cooked metadata/data split is documented
- [ ] tests define expected cooker outputs

### M15.2 Create `lune_cooker` CLI Shell

Type: Scaffold

Blocked by: M15.1

User story: As a tool user, I want a CLI entry point for cooking assets.

Acceptance criteria:

- [ ] `lune_cooker` binary exists
- [ ] CLI args are parsed with `clap`
- [ ] errors use `anyhow` at app boundary and typed errors in libraries

### M15.3 Cook Shaders and Reflection

Type: Learning

Blocked by: M15.2, M8.3

User story: As a renderer, I want shader libraries precompiled with reflection
metadata.

Acceptance criteria:

- [ ] shader source compiles to cached SPIR-V
- [ ] reflection metadata is serialized
- [ ] `insta` snapshots cover output

### M15.4 Cook Meshes and Textures

Type: Learning

Blocked by: M15.2, M10.2

User story: As a runtime loader, I want meshes and textures transformed into
GPU-friendly formats before launch.

Acceptance criteria:

- [ ] mesh layout optimization path exists
- [ ] mip generation/compression plan is implemented or explicitly staged
- [ ] cooked output includes dependency metadata

### M15.5 Add Parser/Importer Fuzzing

Type: Learning

Blocked by: M15.2

User story: As a maintainer, I want malformed inputs to fail safely.

Acceptance criteria:

- [ ] `cargo fuzz` target exists for at least one parser/importer
- [ ] crashes are treated as bugs
- [ ] corpus location and workflow are documented

### M15.6 Audit Test Quality

Type: Learning

Blocked by: M3.7, M2.10

User story: As a maintainer, I want to know whether tests catch meaningful
logic changes.

Acceptance criteria:

- [ ] `cargo-mutants` workflow is documented
- [ ] at least one core crate is mutation-tested
- [ ] weak tests produce follow-up tasks
