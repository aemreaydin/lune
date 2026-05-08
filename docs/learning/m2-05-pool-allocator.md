# M2.5 Pool Allocator

## Status

Scaffolded.

## Goal

This topic teaches how Lune implements a fixed-size pool allocator with explicit
allocate, free, reuse, double-free detection, alignment, capacity, and pool
statistics behavior. It unlocks predictable storage for future fixed-size engine
records such as resource wrappers, command objects, and handle-backed tables.

## Concept

A pool allocator owns a fixed number of fixed-size slots. Allocation removes one
slot from the free set. Free returns that slot so a later allocation can reuse
it. Unlike a linear allocator, a pool does not reclaim everything at once and it
does not support arbitrary allocation sizes after construction.

For Lune M2, the pool remains metadata-only. A successful allocation returns
`PoolAllocation`:

- `slot_index`: which fixed slot was reserved
- `generation`: slot reuse generation
- `offset`: byte offset in the pool's storage model
- `size`: requested slot payload size
- `align`: slot alignment

The generation is what lets the allocator detect copied stale metadata. If slot
0 is allocated, freed, and allocated again, the second allocation can reuse the
same slot and offset, but it should have a newer generation. A later `free` with
the old generation is not valid.

### Fixed Slots

The pool is built from one `MemoryLayout`. Every slot has the same payload size
and alignment. The slot stride may be larger than the payload size because each
slot's offset must satisfy the slot alignment:

```text
slot_stride = align_up(slot_size, slot_align)
slot_offset = slot_index * slot_stride
```

The total byte capacity is `slot_stride * slot_count`. Both calculations need
checked arithmetic. Overflow should be reported as `MemoryError::OutOfMemory`
rather than wrapping.

### Free And Reuse

Freeing a slot must make capacity available without changing the slot's offset.
The next allocation is allowed to reuse the freed slot. The test contract chooses
deterministic reuse of the freed slot so the behavior is easy to reason about.

Double-free is a misuse case. `PoolAllocation` is copyable metadata, so Rust move
semantics cannot prevent all repeated frees. Lune detects the repeated free and
returns `MemoryError::DoubleFree`.

### Stats

Pool stats are separate from `AllocationStats` because pool pressure is easier to
read in slot units:

- `capacity_bytes`
- `slot_size`
- `slot_align`
- `slot_stride`
- `slot_count`
- `active_slots`
- `free_slots`
- `peak_active_slots`
- `successful_allocations`
- `failed_allocations`

`used_bytes` is less useful for pool behavior because a freed slot's bytes still
belong to the pool. Active/free slots show the pressure directly.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Fixed-size pool | Predictable reuse, simple stats, no fragmentation inside one slot size | One pool only fits one layout | Resource records and fixed command objects |
| Slab with optional values | Simple safe Rust representation | Ties allocation to typed storage and `Option` overhead | Higher-level typed storage |
| Free list over variable blocks | Handles many sizes | More fragmentation policy and metadata | General-purpose memory allocator |
| Generational slot map | Strong stale-handle detection | More handle/table design than allocator lesson | ECS entities and asset handles |
| System allocator | General and mature | No local capacity/reuse behavior | Non-hot paths and general containers |

## Industry Examples

Object pools are a common engine pattern for frequently created fixed-shape
objects such as particles, command records, temporary resource wrappers, or
backend descriptors. The core idea is avoiding repeated heap allocation by
reusing slots that are already reserved by the engine.

Rust crates such as `slab` and `slotmap` show adjacent ideas. `slab` stores typed
values in reusable slots, while `slotmap` adds generational keys for stale-handle
detection. Lune keeps M2 lower level and metadata-only so the slot layout,
alignment, stats, and free/reuse rules are visible before typed storage arrives.

## Lune Architecture Impact

`PoolAllocator` lives in `lune_memory` beside the linear and frame allocators.
It should not depend on ECS, renderer, assets, or platform crates.

The current API surface is:

- `PoolAllocator`: owns a fixed slot layout and fixed slot count
- `PoolAllocation`: carries slot, generation, offset, size, and alignment
- `PoolStats`: reports capacity and active/free slot pressure
- `MemoryError::OutOfMemory`: reports full pool or arithmetic overflow
- `MemoryError::DoubleFree`: reports repeated free of copied allocation metadata

The pool uses inherent `allocate` and `free` methods first. It does not need to
implement `Allocator` in M2.5 because the base trait cannot express `free`, and
the pool's allocation request is fixed at construction time. A later trait
redesign can revisit whether pool allocation should be generic.

## Decision

Add a fixed-size, metadata-returning `PoolAllocator` with deterministic slot
reuse, copied allocation metadata, generation-based double-free detection, and
dedicated pool stats. Keep pointer-backed storage and typed object construction
out of M2.5.

## Failing Tests

The tests live in `crates/lune_memory/tests/pool_allocator.rs`.

They define this behavior:

- allocation fills fixed slots until the pool is full
- full pools return typed `MemoryError::OutOfMemory`
- freeing a slot makes it available for reuse
- reused slots keep the same offset and get a newer generation
- double-free returns typed `MemoryError::DoubleFree`
- slot stride preserves alignment for every slot
- stats expose active slots, free slots, peak activity, capacity, and counts

These tests are expected to compile and fail at `todo!()` boundaries until the
production implementation is filled in.

## Implementation Handles

- Key types: `PoolAllocator`, `PoolAllocation`, `PoolStats`, `MemoryLayout`,
  `MemoryError`.
- Key invariants: slot offsets are aligned; a slot is either active or free;
  active plus free slots equals slot count; reused slots advance generation;
  failed allocations do not change active/free slots.
- Error cases: full pool, stride/capacity arithmetic overflow, double-free.
- Edge cases: zero slots, one-slot pools, freeing then reusing, copied stale
  allocation metadata, slot sizes that need padding to preserve alignment.
- Useful assertions: `offset == slot_index * slot_stride`,
  `offset % align == 0`, `free_slots + active_slots == slot_count`, and
  `peak_active_slots` never decreases.

## Verification

Run the pool allocator tests directly while working on this topic:

```bash
cargo test -p lune_memory --test pool_allocator
```

Run the full local verification command after implementation:

```bash
just check
```

Miri is not required for this metadata-only implementation. Add it once pool
slots own typed values, raw storage, or drop behavior:

```bash
just miri
```

## Follow-Up Topics

- M2.6 small game containers explanation
- M2.7 fixed-capacity collection APIs
- M3.1 generational handles
- typed pool storage and destructor policy

## Additional Reading

- Rust `std::alloc::Layout`: https://doc.rust-lang.org/stable/std/alloc/struct.Layout.html
- Game Programming Patterns, Object Pool: https://gameprogrammingpatterns.com/object-pool.html
- `slab` crate documentation: https://docs.rs/slab/latest/slab/
- `slotmap` crate documentation: https://docs.rs/slotmap/latest/slotmap/
- Ginger Bill, memory allocation strategies: https://www.gingerbill.org/series/memory-allocation-strategies/
