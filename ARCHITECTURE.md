# Lune Architecture

Lune is a learning-first 3D game engine written in Rust. The project goal is
to understand industry-standard engine architecture by implementing the core
systems directly, with AI acting as a guide, test author, and reviewer.

## Non-Negotiable Workflow

- The user owns production implementations for learning topics.
- AI may write explanations, design docs, failing tests, crate skeletons, API
  signatures, `todo!()` stubs, and non-learning glue.
- AI must not implement production logic for learning topics unless explicitly
  asked.
- For each topic, AI explains the concept, alternatives, industry examples,
  and Lune architecture impact before writing tests.
- AI-written tests should be comprehensive enough to define the expected
  behavior, not just prove the happy path. Cover defaults, builders or
  constructors, success cases, typed error cases, edge cases, and regression
  risks that are practical for the topic.
- Tests should compile and fail at runtime where possible. Compile-failing API
  sketches are allowed only when clearly marked.
- Learning docs should start from [`docs/learning/TEMPLATE.md`](docs/learning/TEMPLATE.md).
- Crate `lib.rs` files should stay focused: module declarations, public
  re-exports, crate-level docs, crate error/result types, and the crate's core
  implementation entry points may live there. Supporting structs, enums, and
  use-case-specific types should move into modules unless they are tightly
  coupled to the crate root.
- Before starting a topic, read this file, the decision ledger, and relevant
  ADRs.

## Engine Direction

Lune is modular and library-heavy. It starts on Linux, uses Vulkan through a
clean render API, runs on a custom ECS architecture, and supports hot reloading
for assets and scripts after the initial static scene showcase.

The first real showcase is `showcase_01_static_scene`:

- open a window
- create a Vulkan renderer
- render a glTF static scene
- use ECS entities with transforms, meshes, and materials
- provide a free/debug camera and scene camera support
- show basic diagnostics
- no hot reload yet

Renderer bring-up uses one permanent `lune_renderer_smoke` showcase with modes
such as clear, triangle, and textured cube. Headless/offscreen renderer tests
come later.

## Workspace Layout

```text
crates/
  lune_core/
  lune_math/
  lune_memory/
  lune_collections/
  lune_config/
  lune_perf/
  lune_diagnostics/
  lune_debug/
  lune_platform/
  lune_input/
  lune_render/
  lune_render_vulkan/
  lune_asset/
  lune_scene/
  lune_scene_gltf/
  lune_script/
  lune_script_lua/
  lune_physics/
  lune_physics_rapier/
  lune_animation/
  lune_audio/
  lune_audio_kira/
  lune_task/
  lune_cooker/
showcases/
  lune_renderer_smoke/
  showcase_01_static_scene/
docs/
  adr/
  learning/
assets/
  showcases/
tests/
  fixtures/
```

`xtask` is added later when repo automation needs Rust logic. A `justfile` is
used first for command aliases.

## Crate Layers

Base crates:

- `lune_config`: TOML config types and defaults/root/showcase merge semantics
- `lune_diagnostics`: typed error/logging setup, tracing subscriber helpers
  that consume diagnostics config from `lune_config`
- `lune_memory`: allocators, arenas, pools, frame allocators, handle allocators
- `lune_collections`: custom small game-oriented containers
- `lune_math`: `glam` re-export and engine math helpers

Core/runtime crates:

- `lune_core`: entities, ECS, resources, events, app, schedule
- `lune_perf`: frame stats, counters, profiling markers, benchmark helpers
- `lune_task`: job system and worker infrastructure
- `lune_platform`: window/platform integration
- `lune_input`: input abstraction over platform events

Asset/scene/simulation crates:

- `lune_asset`: asset registry, typed handles, loader traits
- `lune_scene`: runtime scene data and ECS spawning
- `lune_scene_gltf`: glTF import
- `lune_script`: gameplay scripting facade
- `lune_script_lua`: Lua backend
- `lune_physics`: physics facade
- `lune_physics_rapier`: Rapier backend
- `lune_animation`: custom animation runtime
- `lune_audio`: audio facade
- `lune_audio_kira`: Kira backend

Rendering/tooling crates:

- `lune_render`: render API, resource descriptions, capability model
- `lune_render_vulkan`: Vulkan backend implementation
- `lune_debug`: debug tools, overlays, inspectors, debug draw
- `lune_cooker`: future asset cooking CLI/library

Dependency direction is one-way. Low-level crates do not depend on high-level
engine concepts. Vulkan types do not leak outside `lune_render_vulkan`.

## Decision Ledger

| Area | Decision | Status | Rationale | ADR |
| --- | --- | --- | --- | --- |
| Project | Name is Lune | Accepted | Modern, minimal, short crate prefix | none |
| Workflow | AI writes docs/tests/stubs, user implements learning topics | Accepted | Preserves learning ownership | ADR-0001 |
| Rust | Edition 2024, pin current stable Rust, initially 1.95.0 | Accepted | Modern Rust learning target | none |
| Workspace | Single Rust workspace | Accepted | Easier refactors and integrated showcases | ADR-0002 |
| Crates | Use `lune_*` focused crates | Accepted | Library-heavy modular design | ADR-0002 |
| Features | No Cargo features initially | Accepted | Avoid feature-matrix complexity | none |
| Config | Defaults + root `Lune.toml` + showcase override | Accepted | Simple mental model; showcase wins | none |
| CLI/runtime config | Future layers, not first implementation | Accepted | Useful later without early complexity | none |
| Errors | Libraries use `thiserror`; apps/tools may use `anyhow` | Accepted | Typed library errors, ergonomic app glue | none |
| Logging | Engine code uses `tracing`; bridge `log` via `tracing-log` if needed | Accepted | Structured diagnostics | none |
| Memory | `lune_memory` from day one with simple allocators | Accepted | Memory is core learning topic | ADR-0003 |
| Memory usage | Prefer Lune allocator APIs in engine-owned paths; `Vec`/`Box` allowed | Accepted | Learn memory without unsafe everywhere | ADR-0003 |
| Collections | Custom `lune_collections` from start, small scope, heavy tests | Accepted | Game containers are learning topic | ADR-0003 |
| Helper crates | `bytemuck`/`zerocopy` allowed after POD/layout lesson | Accepted | Useful safety boundary for binary/GPU data | ADR-0003 |
| Reference crates | `smallvec`, `arrayvec`, `slotmap`, ECS crates, GPU allocators are references only | Accepted | Do not replace learning implementations | ADR-0003 |
| ECS | Custom archetype ECS | Accepted | Industry-relevant, performance-focused learning | ADR-0003 |
| Entity IDs | Generational entity handles | Accepted | Detect stale references; common handle-table pattern | ADR-0003 |
| Components | Static typed Rust components first | Accepted | Type safety and simpler performance model | ADR-0003 |
| ECS API | Low-level internals first, ergonomic typed API later | Accepted | Learn storage before polishing API | ADR-0003 |
| Resources | Minimal ECS resources from start | Accepted | Engine systems need global typed state | ADR-0003 |
| Systems | Explicit `System` trait first, function-param systems later | Accepted | Avoid early macro/type complexity | ADR-0003 |
| Events | Typed event queues as resources | Accepted | Type-safe, testable, ECS-friendly | ADR-0003 |
| Hierarchy | Runtime ECS hierarchy with `LocalTransform`, `GlobalTransform`, `Parent`, `Children` | Accepted | ECS is runtime truth; scene layer spawns data | ADR-0003 |
| Transform update | Full hierarchy recompute first, dirty propagation later | Accepted | Correctness before optimization | none |
| Scheduling | Scheduler abstraction day one; job system early later milestone | Accepted | Important learning topic but not first showcase blocker | ADR-0003 |
| Main loop | Hybrid timestep: variable update, fixed physics, render every frame | Accepted | Common engine pattern | none |
| Threading | Single engine thread first; backend-owned threads allowed | Accepted | Keep ownership tractable, leave extract boundary | none |
| Math | Right-handed, Y-up, forward `-Z`, meters, radians | Accepted | glTF-compatible conventions | none |
| Rendering | `lune_render` RHI-style API; Vulkan backend separate | Accepted | API isolation and backend optionality | ADR-0004 |
| Vulkan exposure | No Vulkan types outside `lune_render_vulkan` | Accepted | Keeps engine backend-neutral | ADR-0004 |
| Vulkan target | Vulkan 1.2 baseline; prefer Vulkan 1.4/modern extensions | Accepted | Modern-first but runs on common hardware | ADR-0004 |
| Modern/fallback | Implement modern path and fallback together per feature | Accepted | Testability and real fallback confidence | ADR-0004 |
| Capability forcing | Runtime/config support for forcing modern/fallback paths | Accepted | Needed to test both paths | ADR-0004 |
| First render path | Forward renderer first | Accepted | Smallest useful renderer path | ADR-0004 |
| Later render paths | Deferred, forward+, and other paths later | Accepted | Learn alternatives incrementally | ADR-0004 |
| Shader objects | Implement with classic pipeline fallback when introduced | Accepted | Must respect modern+fallback rule | ADR-0004 |
| Sync | Timeline/sync2 modern path plus binary/legacy fallback | Accepted | Core Vulkan learning topic | ADR-0004 |
| Render graph | Design early; implement after forward renderer works | Accepted | Avoid blocking first visual progress | ADR-0004 |
| Descriptors | Classic descriptor sets first; bindless later | Accepted | Baseline Vulkan first, modern resource model later | ADR-0004 |
| Shaders | Start with Slang, compile to SPIR-V, use reflection | Accepted | Modern shader workflow and reflection learning | ADR-0004 |
| Shader pipeline | Dev compile/cache first; offline cooker later | Accepted | Staged path to real asset pipeline | ADR-0004 |
| Vulkan debug | Validation, debug utils labels/names, shader printf if supported | Accepted | Deep Vulkan diagnostics from start | ADR-0004 |
| Vulkan shutdown | RAII wrappers plus explicit ordered shutdown/deletion queues | Accepted | Vulkan lifetime correctness is critical | ADR-0004 |
| Vulkan memory | Custom simple GPU allocator from start, improve over time | Accepted | Vulkan memory is learning priority | ADR-0004 |
| Render/ECS boundary | Extract render data before rendering | Accepted | Future render thread and clean ownership | ADR-0004 |
| Mesh layout | Interleaved vertex first; RHI allows separate streams later | Accepted | Simple first, flexible future | ADR-0004 |
| Initial vertex | Position, normal, tangent, UV0 | Accepted | Supports PBR-lite and normal maps | ADR-0004 |
| Materials | Fixed PBR-lite material first, graph later | Accepted | glTF-friendly without editor complexity | ADR-0004 |
| Lighting | Directional light + ambient, no IBL/shadows initially | Accepted | Validates 3D materials without full PBR cost | ADR-0004 |
| Textures | PNG/JPEG via `image`, RGBA8, correct sRGB/linear handling | Accepted | glTF basics and color correctness | none |
| Assets | Per-type generational `Handle<T>` runtime handles | Accepted | Type-safe, hot-reload-friendly | ADR-0005 |
| Asset IDs | Stable global asset DB IDs later | Accepted | Useful for editor/cooker but not runtime first | ADR-0005 |
| Asset load | Synchronous first, API shaped for async later | Accepted | Deterministic early implementation | ADR-0005 |
| Hot reload | After initial showcase, via watcher and reload queue | Accepted | Important feature, not first slice | ADR-0005 |
| Reload failure | Keep old good asset, report error | Accepted | Iteration-friendly behavior | ADR-0005 |
| Scene import | glTF first | Accepted | Standard real asset interchange | ADR-0005 |
| Scene format | Lune `.ron` scene/import/cache metadata later | Accepted | glTF is asset input, not full game scene | ADR-0005 |
| Metadata formats | TOML for config, RON for scene/import/cache metadata | Accepted | Human config plus Rust-friendly internal data | none |
| Scripting | Lua via `mlua` | Accepted | Industry-familiar embeddable language | ADR-0005 |
| Script API | Safe gameplay facade + command API, no raw ECS mutation | Accepted | Preserve ECS invariants | ADR-0005 |
| Lua VM | One VM per app/world, isolated script environments | Accepted | Shared bindings with script isolation | ADR-0005 |
| Script reload | Restart script instance first; state migration later | Accepted | Predictable first hot reload | ADR-0005 |
| Physics | Facade plus Rapier backend | Accepted | Physics internals are lower priority | none |
| Animation | Custom runtime, glTF-backed | Accepted | Important manageable engine learning topic | none |
| Audio | Facade plus Kira backend | Accepted | Audio internals are low priority | none |
| Debug UI | Immediate-mode debug UI first, likely egui behind abstraction | Accepted | Fits debug overlays; editor can revisit | none |
| Editor | Full editor later | Accepted | Debug tools first | none |
| Tests | Unit tests most important; integration/golden at milestone boundaries | Accepted | Supports learning loop | ADR-0001 |
| Test scope | Public behavior tests plus internal learning tests | Accepted | Protect API and teach internals | ADR-0001 |
| Test names | Explicit behavior names | Accepted | Readable failing tests | ADR-0001 |
| Golden tests | Snapshot/import/render-graph first; image golden tests much later | Accepted | Avoid fragile GPU output early | ADR-0001 |
| CI | Local-only first; static CI later; smoke CI near end | Accepted | Avoid fragile early rendering CI | none |
| Unsafe | Safe Rust default; unsafe only low-level crates | Accepted | Contain invariants | ADR-0003 |
| High-level unsafe | High-level libs and showcases must not use unsafe | Accepted | Keep API safety boundary clear | ADR-0003 |
| Dependencies | Latest stable crates pinned by `Cargo.lock`, upgraded intentionally | Accepted | Modern but controlled | none |
| Tooling | Use full cargo tool list by milestone | Accepted | Tooling introduced when relevant | none |
| `just` | Use `justfile` day one | Accepted | Simple command aliases | none |
| `xtask` | Add later for real Rust automation | Accepted | Good for shader/cooker/docs logic | none |
| Cooker | `lune_cooker` separate from `xtask` | Accepted | Engine tool, not repo-only automation | none |
| License | `MIT OR Apache-2.0` | Accepted | Rust ecosystem standard permissive license | none |
| Assets | Small assets committed, large external packs later | Accepted | First showcase should run without downloads | none |
| Platform | Linux first, Windows second, macOS/MoltenVK later if worth it | Accepted | Vulkan learning path first | none |

## Open Decisions

- Final GUI crate for debug UI and future editor. `egui` is likely for early
  immediate-mode debug UI but must be evaluated for theming, Vulkan integration,
  and performance.
- Exact public API shape for typed ECS queries and system parameters.
- Exact Slang integration path: Rust bindings, direct SDK, or external `slangc`
  process.
- Exact file format for cooked binary assets.
- When to keep, delete, or convert specific temporary renderer smoke modes.

## Third-Party Crate Baseline

Initial or planned crates:

- Vulkan: `ash`
- window/events: `winit`
- math: `glam`
- errors: `thiserror`, `anyhow` only in apps/tools
- logging/instrumentation: `tracing`, `tracing-subscriber`, `tracing-log`
- config/metadata: `serde`, `toml`, `ron`
- glTF: `gltf`
- images: `image`
- Lua: `mlua`
- audio backend: `kira`
- physics backend: `rapier3d`
- file watching later: `notify`
- CLI tools later: `clap`
- tests: `pretty_assertions`, `proptest`, `insta`
- low-level binary/POD helpers after lesson: `bytemuck`, `zerocopy`

Use learning-area helper crates only when they do not replace the learning
objective.

## Development Tools

Foundation:

- `cargo fmt`
- `cargo clippy`
- `cargo nextest`
- `cargo test --doc`
- `bacon`
- `cargo tree -d`

Dependency hygiene:

- `cargo-deny`
- `cargo-machete`

Memory, collections, ECS:

- `cargo miri`
- `proptest`
- `criterion`

Snapshots and golden outputs:

- `cargo insta`

Performance:

- `cargo flamegraph`
- `cargo bloat`
- `iai-callgrind`

Concurrency and hardening:

- `loom`
- `cargo fuzz`
- `cargo-mutants`

Later API/tool introspection:

- `cargo-expand`
- `cargo-semver-checks`

## Topic Done Criteria

A topic is done only when:

- learning doc exists
- alternatives and tradeoffs are documented
- industry examples are included
- relation to Lune architecture is documented
- AI-written failing tests exist and cover success paths, failure paths, edge
  cases, and public API contracts that are practical for the topic
- user implementation makes tests pass
- public API is reviewed
- stale scaffolding is removed or explicitly retained
- this decision ledger or an ADR is updated if a decision changed
