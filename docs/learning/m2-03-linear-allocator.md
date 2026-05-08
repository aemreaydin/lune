# M2.3 Linear Allocator

## Status

Implemented.

## Goal

This topic teaches how Lune implements a simple bump allocator with explicit
alignment, capacity, reset, and statistics behavior. It unlocks the concrete
memory provider that later frame, renderer scratch, and arena-like APIs can wrap.

## Concept

A linear allocator owns a fixed byte capacity and a cursor. Allocation is a
single forward move:

1. align the cursor to the requested alignment
2. reserve the requested byte count
3. advance the cursor to the end of the reserved range

There is no individual free. Reclaiming memory is a bulk operation: `reset`
moves the cursor back to zero and invalidates all previous allocations.

For Lune M2, the allocator returns metadata instead of raw pointers. The
metadata still has to obey the same shape that pointer-backed allocations will
need later:

```text
allocation.offset % allocation.align == 0
allocation.offset + allocation.size <= allocator.capacity_bytes
```

The key implementation risk is arithmetic. Alignment can require padding, and
`cursor + padding + size` can overflow before a normal capacity comparison
catches it. The linear allocator must use checked arithmetic and report overflow
as `MemoryError::OutOfMemory`.

### Zero-Sized Allocations

Zero-sized allocations are valid. They should produce aligned metadata and count
as successful allocation requests, but they must not advance the cursor or
increase used bytes. This mirrors the idea that a zero-sized value has alignment
requirements but does not reserve storage.

That rule means a zero-sized allocation may return an aligned offset ahead of the
current cursor while the next nonzero allocation still reuses the original
cursor. This is safe in the current metadata-only contract because the
zero-sized allocation owns no byte range.

The aligned metadata offset must still stay inside allocator capacity. If a
zero-sized request would need an aligned offset past capacity, it fails with
`MemoryError::OutOfMemory` without advancing the cursor.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Linear allocator | Very fast allocation, simple reset, easy stats | No individual frees; reset invalidates everything | Frame scratch, transient batches |
| Free-list allocator | Reuses individual blocks | More metadata and fragmentation policy | Variable-size long-lived allocations |
| Pool allocator | Predictable fixed-size reuse | Only fits one slot size unless generalized | Resource records, fixed object pools |
| System allocator | Mature and general | Hides allocation policy and stats from engine code | General Rust containers and non-hot paths |

## Industry Examples

Game engines commonly use linear or arena allocators for temporary work such as
per-frame command building, renderer staging, visibility lists, and short-lived
simulation scratch data. The exact implementations vary, but the common pattern
is the same: allocate cheaply during a bounded phase and reset the whole arena at
a clear lifetime boundary.

Rust arena crates such as `bumpalo` expose a similar lifetime model at a higher
level. Lune keeps M2 lower level and metadata-only so the project can learn the
alignment, overflow, and invalidation rules before adding typed allocation APIs.

## Lune Architecture Impact

`LinearAllocator` is the first concrete allocator in `lune_memory`. It stays in
the base crate and depends only on memory-local types:

- `MemoryLayout` for validated request shape
- `Allocation` for offset, size, and alignment metadata
- `MemoryError` for typed failure
- `AllocationStats` for capacity, usage, peak, and allocation counts
- `Allocator` and `ResettableAllocator` for shared API dispatch

This implementation is intentionally safe Rust. Miri becomes more important once
future topics introduce raw pointers, typed allocation helpers, drop handling, or
unsafe backing-buffer access.

## Decision

Keep `LinearAllocator` as a fixed-capacity, metadata-returning bump allocator.
It aligns each allocation with checked arithmetic, returns typed out-of-memory
errors for capacity and overflow failures, updates success and failure stats, and
uses `reset` to clear current usage while preserving peak and allocation count
history.

Zero-sized allocations return aligned metadata and increment successful
allocation count without advancing the cursor.

Do not add pointer-backed allocation helpers in M2.3. That boundary belongs in a
later topic with explicit lifetime and invalidation tests.

## Contract Tests

The tests live in `crates/lune_memory/tests/allocator_contract.rs` and
`crates/lune_memory/tests/allocator_interface.rs`.

They cover:

- valid allocations start at aligned offsets
- used bytes include padding for nonzero allocations
- exact and near-capacity allocation behavior
- out-of-memory returns a typed error
- failed allocations do not advance the cursor
- reset reuses capacity and preserves peak stats
- zero-sized allocations do not consume capacity
- zero-sized allocations after a nonzero cursor still return aligned metadata
- zero-sized allocations fail if their aligned metadata offset exceeds capacity
- arithmetic overflow during alignment or allocation-end sizing reports
  out-of-memory
- trait dispatch preserves allocator behavior

## Implementation Handles

- Key types: `LinearAllocator`, `Allocation`, `MemoryLayout`,
  `AllocationStats`, `MemoryError`.
- Key invariants: alignment is nonzero and power-of-two; nonzero allocation
  ranges stay inside capacity; failed allocations leave cursor and used bytes
  unchanged; reset invalidates previous allocations.
- Error cases: invalid layout before allocation, capacity exhaustion, alignment
  arithmetic overflow, allocation-end arithmetic overflow.
- Edge cases: zero-sized allocations, padding-only alignment, exact capacity,
  failed allocation followed by a smaller success, reset after prior peak usage,
  zero-sized requests whose aligned metadata would exceed capacity.
- Useful assertions: `used_bytes` equals the cursor for nonzero allocations,
  `peak_used_bytes` never decreases, and zero-sized allocations increment
  success count without changing `used_bytes`.

## Verification

Run the allocator contract tests directly while working on linear allocator
behavior:

```bash
cargo test -p lune_memory --test allocator_contract
```

Run the allocator trait tests when changing shared allocator APIs:

```bash
cargo test -p lune_memory --test allocator_interface
```

Run the full local verification command before closing the topic:

```bash
just check
```

Miri is not required for the current implementation because it has no unsafe or
drop-sensitive memory access. Add it once pointer-backed allocator internals
arrive:

```bash
just miri
```

## Follow-Up Topics

- M2.4 frame allocator wrapper
- M2.5 pool allocator
- M2.7 fixed-capacity collection APIs
- M3.1 generational handles

## Additional Reading

- Rust `std::alloc::Layout`: https://doc.rust-lang.org/stable/std/alloc/struct.Layout.html
- Rust `std::alloc` module: https://doc.rust-lang.org/stable/std/alloc/index.html
- Rustonomicon `Vec` allocation chapter: https://doc.rust-lang.org/nomicon/vec/vec-alloc.html
- `bumpalo` arena allocator documentation: https://docs.rs/bumpalo/latest/bumpalo/
