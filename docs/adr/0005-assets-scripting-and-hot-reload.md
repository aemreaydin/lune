# ADR-0005: Assets, Scripting, and Hot Reload

## Status

Accepted

## Context

Lune should support asset and script hot reload, but the first static scene
showcase should not be blocked by a full asset pipeline.

Assets, scenes, renderer resources, and scripts need stable runtime references
that can survive reloads and detect stale references.

## Decision

Runtime assets use per-type generational handles such as `Handle<Mesh>`,
`Handle<Texture>`, and `Handle<Material>`. Path strings remain source metadata,
not runtime references. Stable global asset IDs are added later for the asset
database, editor, dependency graph, and cooker.

Initial loading is synchronous. APIs are shaped so async/background loading can
be added later through `lune_task` and a render upload queue.

glTF is the first asset interchange format. Lune scene/import/cache metadata
uses RON, while human-authored configuration uses TOML. A Lune scene format is
added after direct glTF loading, so scenes can reference assets, active cameras,
lights, script bindings, and overrides.

Hot reload is added after the first static scene showcase. The first design uses
filesystem watching, an asset registry, reload queues applied on the main
thread, and safe renderer/script resource replacement. If reload fails, the old
good asset remains live and the error is reported through diagnostics.

Lua is the first scripting language via `mlua`. Scripts interact with the
engine through a safe gameplay facade and command API. Scripts must not mutate
raw ECS internals. Lune uses one Lua VM per app/world with isolated script
environments. Reloading a script restarts the script instance first; state
migration can be added later.

## Consequences

The runtime reference model is ready for hot reload without requiring the full
cooker/editor pipeline up front. The scripting boundary protects ECS invariants
while still allowing gameplay iteration.
