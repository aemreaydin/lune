# M3.1 Generational Handles

## Status

Implemented.

## Goal

Understand why engine handles usually store both an index and a generation.
This milestone sets the contract for `Entity` handles and the entity slot table
that validates them.

## Concept

A plain integer ID can find a slot, but it cannot tell whether that slot still
contains the same object. If slot `5` is despawned and later reused for a new
entity, an old copied handle that only stores `5` would accidentally refer to
the new entity.

A generational handle stores:

- `index`: which slot to inspect.
- `generation`: which lifetime of that slot the handle refers to.

The slot table stores the current generation beside each slot. A handle is live
only when its index exists, the slot is occupied, and the handle generation
matches the slot generation.

### Free-List Pattern

The entity table owns a vector of slots and a free list of reusable indexes.

Spawn behavior:

- If the free list has an index, reuse that slot.
- Otherwise append a new slot.
- Mark the slot occupied.
- Return `Entity { index, generation }`.

Despawn behavior:

- Reject generation `0` handles.
- Reject indexes outside the slot table.
- Reject handles whose generation does not match the current slot generation.
- Reject handles whose generation matches a slot that is already free.
- Mark the slot free.
- Increment its generation so old handles become stale.
- Push the index onto the free list.

The first live generation is `1`. Generation `0` is reserved for null or invalid
handles. Reusing a slot increments the generation before the next live handle is
issued.

### Invalid And Overflow Policy

Invalid handles are ordinary API errors, not panics:

- `EntityError::NullEntity` for generation `0`.
- `EntityError::InvalidEntity` for an index that has never belonged to the
  table.
- `EntityError::StaleEntity` for a valid index with the wrong generation.
- `EntityError::SlotFree` for a synthesized handle whose generation matches a
  slot that is currently unoccupied. This case is unreachable through normal
  `spawn`/`despawn` flows, but the variant exists so the error never lies
  about generation mismatch when no mismatch occurred.

Generation overflow is rare but must be explicit. If incrementing a slot
generation would overflow `u32`, that slot must become unavailable and the API
returns `EntityError::GenerationOverflow { index }`. The table must not reuse
a slot with a wrapped generation, because that would make stale handles look
live again.

Index exhaustion is also explicit. Live entity indexes occupy the range
`0..u32::MAX`. Index `u32::MAX` is reserved as the index slot for `Entity::NULL`
and must never be returned by `spawn`. If appending a new slot would reach that
limit, `spawn` returns `EntityError::IndexExhausted { capacity }`.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Plain `u32` ID | Smallest handle, simplest table | Cannot detect stale handles after reuse | Throwaway scripts, single-frame IDs that are never copied |
| Index + generation handle | Detects stale handles in O(1); identity is stable across copies | Slightly larger handle; needs free list and generation tracking | General ECS entity identity, asset handles, gameplay objects |
| `slotmap::SlotMap` (reference crate) | Mature, well-tested, several flavors | External dependency; opaque internals for learning | Production code once the learning step is done |
| Pointer-only handle (`*const T`) | Direct dereference, no table lookup | Lifetime headaches across systems; no stale detection without extra metadata | Local short-lived references inside one system |

## Industry Examples

Generational handles are the dominant pattern for entity and asset identity in
modern Rust and C++ engines.

- Bevy ECS represents entities as an index + generation pair, validated through
  its entity meta table.
- EnTT (C++) uses an index + version encoding inside a single integer for
  entity identity.
- Unity DOTS exposes `Entity` as an index + version struct with the same
  stale-handle detection contract.
- Unreal's `FObjectHandle` and `TWeakObjectPtr` use serial numbers conceptually
  similar to a generation counter.
- The Rust `slotmap` and `generational-arena` crates encode this pattern as
  reusable libraries.

Lune treats these as references rather than dependencies because index +
generation accounting is the learning target here.

## Lune Architecture Impact

`Entity` and `EntitySlotTable` belong in a new `lune_core` crate that holds
engine-wide identity primitives. The crate must not depend on the renderer,
ECS implementation, platform, or asset crates — those crates depend on it.

Future likely consumers include:

- M3.3 archetype storage, which uses live entities as the identity layer under
  component columns.
- M6 asset handles, which reuse the same index + generation pattern for typed
  runtime assets.

## Decision

Add `lune_core` with an `Entity` handle and `EntitySlotTable`:

```rust
pub struct Entity;

impl Entity {
    pub const NULL: Entity;
    pub const fn new(index: u32, generation: u32) -> Self;
    pub const fn index(self) -> u32;
    pub const fn generation(self) -> u32;
    pub const fn is_null(self) -> bool;
}

pub struct EntitySlotTable;

impl EntitySlotTable {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn spawn(&mut self) -> EntityResult<Entity>;
    pub fn despawn(&mut self, entity: Entity) -> EntityResult<()>;
    pub fn is_alive(&self, entity: Entity) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn capacity(&self) -> usize;
}

impl Default for EntitySlotTable { ... }
```

`Entity::NULL` uses `index: u32::MAX, generation: 0`. Pairing both invariants
makes accidental collisions impossible: `IndexExhausted` prevents `spawn` from
ever returning index `u32::MAX`, and `is_null` checks the reserved generation
`0` so any `Entity::new(_, 0)` is also treated as null.

The overflow boundary cannot be reached through public APIs in a reasonable
test (it requires `u32::MAX` despawn cycles), so overflow-policy tests live
inline in `src/entity.rs` as `#[cfg(test)] mod tests` and seed the slot table
through its private fields. The public API stays free of hidden test hooks.

## Tests

The M3.1/M3.2 tests define and verify the slot-table behavior:

```rust
#[test]
fn entity_handle_exposes_index_generation_and_reserved_null() { /* ... */ }

#[test]
fn spawn_creates_a_live_entity() { /* ... */ }

#[test]
fn despawn_invalidates_the_old_handle() { /* ... */ }

#[test]
fn despawning_the_same_handle_twice_is_rejected() { /* ... */ }

#[test]
fn recycled_slot_reuses_index_and_increments_generation() { /* ... */ }

#[test]
fn stale_handle_cannot_despawn_reused_slot() { /* ... */ }

#[test]
fn generation_zero_handles_are_rejected() { /* ... */ }

#[test]
fn foreign_or_unallocated_index_is_rejected() { /* ... */ }

#[test]
fn synthesized_handle_for_free_slot_returns_slot_free() { /* ... */ }

#[test]
fn free_list_reuses_lowest_available_index_deterministically() { /* ... */ }

#[test]
fn capacity_reflects_allocated_slot_storage_not_live_count() { /* ... */ }

#[test]
fn generation_overflow_returns_overflow_error() { /* ... */ }
```

The covered behaviors are:

- handles expose index and generation
- generation `0` is reserved for null/invalid handles
- spawn creates live generation-`1` entities
- despawn invalidates old handles
- double despawn is rejected as stale
- recycled indexes increment generation
- stale handles cannot despawn reused slots
- foreign indexes are rejected
- a synthesized handle matching a freed slot's generation returns `SlotFree`
- the free list reuses the lowest available index deterministically
- capacity reports allocated slot storage, not live entity count
- a slot pre-seeded near `u32::MAX` returns `GenerationOverflow` on the
  despawn that would wrap it

`IndexExhausted` is documented as part of the contract but is not exercised by
a runtime test because spawning `u32::MAX - 1` slots is not practical. M3.2 is
still required to honor it.

## Implementation Handles

- Key types: `EntitySlot { generation: u32, occupied: bool }`, stored in a
  `Vec<EntitySlot>` inside `EntitySlotTable`.
- Key invariants: the first live generation is `1`; an occupied slot's
  generation never changes until despawn; despawn always advances generation
  before pushing the index to the free list.
- Free list: pick a deterministic structure. `BTreeSet<u32>` is simple and
  makes lowest-index reuse straightforward; a stack-based free list also works
  if the documented reuse policy changes. Either is acceptable as long as the
  free-list ordering test pins the chosen behavior.
- Error cases: null handle, out-of-range index, generation mismatch, free
  slot with matching generation, generation overflow, index exhaustion.
- Useful assertions: after `despawn`, the slot at `entity.index()` is
  unoccupied and its generation is `entity.generation() + 1`; the next spawn
  on that index returns the post-increment generation.
- Testing seam: overflow-policy tests live inline in `src/entity.rs` and
  construct an `EntitySlotTable` directly through its private fields, with
  one slot whose generation is pre-advanced to the desired value. Public
  callers cannot reach the overflow boundary, and no test-only constructor
  appears in the public API.

## Relationship To Previous Work

M2.5 `PoolAllocator` already used generation counters to detect stale
allocation metadata. Entity handles apply the same idea to gameplay object
identity:

- pool allocation: index identifies storage for bytes
- entity handle: index identifies storage for ECS metadata

The stale-handle problem is the same in both cases. The difference is that
entities are public-facing engine identity, so their invalid handle policy must
be especially explicit.

## Verification

Run the entity checks:

```bash
cargo test -p lune_core
cargo clippy -p lune_core --all-targets -- -D warnings
cargo fmt --check
```

## Follow-Up Topics

- M3.3 uses valid entities as the identity layer under archetype storage.
- M6 asset handles reuse the same index/generation stale-handle pattern for
  typed runtime assets.

## Additional Reading

- Bevy `Entity` source: https://docs.rs/bevy_ecs/latest/bevy_ecs/entity/struct.Entity.html
- `slotmap` crate: https://docs.rs/slotmap/latest/slotmap/
- `generational-arena` crate: https://docs.rs/generational-arena/latest/generational_arena/
- EnTT entity encoding: https://github.com/skypjack/entt/wiki/Entity-Component-System
- Niklas Frykholm, "Managing Decoupling Part 4 — The ID Lookup Table":
  https://bitsquid.blogspot.com/2014/09/building-data-oriented-entity-system.html
