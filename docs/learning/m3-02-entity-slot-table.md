# M3.2 Entity Slot Table

## Status

Implemented.

## Goal

Implement the storage behind `Entity` so ECS identity can be validated in O(1).
This milestone turns the generational-handle contract from M3.1 into a concrete
slot table that can support future archetype storage.

## Concept

An entity slot table is a small identity allocator. It does not store
components. It stores enough metadata to answer one question reliably:

> Does this copied `Entity { index, generation }` still name a currently live
> entity?

The table owns slots indexed by `Entity::index()`. Each slot tracks whether it
is occupied and which generation is current for that slot. A copied handle is
valid only when:

- the handle generation is not `0`
- the index is in range
- the slot is occupied
- the handle generation equals the slot generation

Despawn makes an old handle stale by freeing the slot and advancing the
generation. Later spawn may reuse the same index, but it returns the newer
generation, so stale handles cannot accidentally affect the new entity.

## Storage Shape

Use one slot record per possible live entity:

```rust
struct EntitySlot {
    generation: u32,
    occupied: bool,
}

pub struct EntitySlotTable {
    slots: Vec<EntitySlot>,
    free: BTreeSet<u32>,
    len: usize,
}
```

`BTreeSet<u32>` is intentionally simple. It gives deterministic lowest-index
reuse, which keeps tests stable and makes traces easier to read. A stack free
list would also be valid engine design, but it would change the documented
reuse policy and the tests.

## Spawn Flow

`spawn` has two paths:

1. If `free` contains an index, remove the lowest free index, mark that slot
   occupied, increment `len`, and return the slot's current generation.
2. Otherwise append a new occupied slot with generation `1`, increment `len`,
   and return the new index.

Generation `0` is reserved for null and invalid handles, so a newly appended
slot starts at generation `1`.

Index `u32::MAX` is reserved for `Entity::NULL`. `spawn` must never return it.
If appending would require that index, return
`EntityError::IndexExhausted { capacity }`.

## Despawn Flow

`despawn(entity)` should validate in this order:

1. If `entity.generation() == 0`, return `EntityError::NullEntity`.
2. If the index is outside `slots`, return `EntityError::InvalidEntity`.
3. If the slot generation differs from the handle generation, return
   `EntityError::StaleEntity`.
4. If the slot is free and the generation matches, return
   `EntityError::SlotFree`.
5. Increment the slot generation with checked arithmetic.
6. Mark the slot free, push the index into `free`, decrement `len`, and return
   `Ok(())`.

If checked generation increment fails, return
`EntityError::GenerationOverflow { index }`. The slot must not wrap to `0`.
Keep the failure behavior conservative: do not report success if the stale
handle prevention generation could not advance.

## Liveness

`is_alive(entity)` is a non-throwing validation helper. It should return `false`
for null, foreign, stale, and free-slot handles. It should not mutate the table.

This method is useful for higher-level ECS code that wants a cheap predicate.
Mutation APIs such as `despawn` should still return detailed errors.

## Testing Seam

The overflow boundary is unreachable through public APIs in any practical test
(it takes `u32::MAX` despawn cycles), but the policy still has to be covered.
Keep those tests inline in `src/entity.rs` under `#[cfg(test)] mod tests`,
where they can construct an `EntitySlotTable` directly through its private
fields with one slot whose generation is pre-advanced to the desired value.
The public API stays free of test-only constructors or `#[doc(hidden)]`
helpers.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| `Vec<EntitySlot>` + `BTreeSet<u32>` | Simple, deterministic reuse, easy tests | `BTreeSet` has log-time free-list operations | Learning implementation and debuggable identity allocator |
| `Vec<EntitySlot>` + stack free list | O(1) reuse, compact metadata | Reuse order depends on despawn order | Production ECS if deterministic lowest-index reuse is not required |
| Packed `u64` entity ID | Smaller API surface, easy hashing | Harder to inspect and teach; still needs slot metadata | Later optimization after contracts are stable |
| `slotmap` crate | Mature implementation | Skips the learning target | Future replacement if Lune stops hand-rolling this layer |

## Lune Architecture Impact

`EntitySlotTable` belongs in `lune_core`, not in future archetype storage. It is
the identity layer other ECS pieces validate against.

Future milestones should build on it this way:

- M3.3 archetype storage assumes entity handles are already stale-safe.
- M3.5 entity location tables can store `Entity` keys without owning identity.
- M6 asset handles reuse the same index/generation policy for typed assets.

The table should remain component-agnostic. Do not add component storage,
querying, archetype IDs, names, hierarchy, or resources in this milestone.

## Decision

Implement `EntitySlotTable` in `crates/lune_core/src/entity.rs` using:

- `Vec<EntitySlot>` for slot metadata
- `BTreeSet<u32>` for deterministic free-index reuse
- `len: usize` for the live entity count
- generation `1` for newly appended slots
- generation `0` as reserved null/invalid
- explicit error variants for null, invalid, stale, free, overflow, and index
  exhaustion cases

Keep `Entity` as a copyable value type with private fields and accessor methods.
Callers should not be able to mutate an entity handle in place.

## Tests

The existing M3.1 tests are the M3.2 acceptance tests:

```bash
cargo test -p lune_core --test entity_slot_table
```

They cover:

- spawn creates live entities
- despawn invalidates old handles
- double-despawn is rejected
- recycled indexes increment generation
- stale handles cannot despawn reused slots
- null and foreign handles are rejected
- synthesized handles for free slots return `SlotFree`
- free-list reuse is deterministic
- capacity is distinct from live count
- generation overflow is explicit

The tests pass against the current `EntitySlotTable` implementation.

## Implementation Notes

- The internal `EntitySlot` type lives near `EntitySlotTable`.
- `with_capacity` allocates slot metadata capacity but keeps `len() == 0`.
- `capacity` returns slot storage capacity, not live entity count.
- `len` counts only occupied slots.
- Do not expose mutable slot references from the public API.
- Convert `u32` indexes to `usize` with `as usize` only after range checks.
- `Entity::NULL` is impossible to produce from `spawn`.
- `Default` should behave like `EntitySlotTable::new()`.

Useful assertions while implementing:

```rust
debug_assert!(self.len <= self.slots.len());
debug_assert!(self.free.len() <= self.slots.len());
```

## Verification

Run the focused checks:

```bash
cargo test -p lune_core
cargo clippy -p lune_core --all-targets -- -D warnings
cargo fmt --check
```

Run Miri for the identity invariants:

```bash
cargo +nightly miri test -p lune_core --test entity_slot_table
```

## Follow-Up Topics

- M3.3 explains archetype ECS storage.
- M3.4 introduces component type metadata.
- M3.5 connects entities to archetype row locations.
- M6 typed asset handles reuse this stale-handle pattern.

## Additional Reading

- Bevy ECS entities: https://docs.rs/bevy_ecs/latest/bevy_ecs/entity/struct.Entity.html
- `slotmap` crate: https://docs.rs/slotmap/latest/slotmap/
- `generational-arena` crate: https://docs.rs/generational-arena/latest/generational_arena/
- EnTT entity docs: https://github.com/skypjack/entt/wiki/Entity-Component-System
