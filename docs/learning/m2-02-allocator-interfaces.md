# M2.2 Allocator Interfaces

## Status

Implemented.

## Goal

This topic teaches how Lune exposes a small allocator contract without exposing
raw pointer ownership too early. It unlocks a common API that future frame, pool,
arena, and renderer staging allocators can implement while preserving typed
failure behavior and allocation metadata.

## Concept

An allocator interface is a promise between a subsystem and a memory provider.
The subsystem should not need to know whether bytes come from a linear allocator,
a pool, a frame arena, or a later GPU staging allocator. It should know only the
shape of a request, the shape of a successful allocation, and the errors it must
handle.

For M2.2, Lune's allocation request is `MemoryLayout`:

- `size`: requested byte count
- `align`: required byte alignment

The allocation result is `Allocation`:

- `offset`: byte offset inside the allocator-owned storage model
- `size`: number of reserved bytes
- `align`: alignment used for the request

The result intentionally carries metadata, not a pointer. That keeps this
milestone focused on allocator math, capacity, reset behavior, and stats. Raw
pointers and typed references need stronger lifetime and invalidation rules. They
belong in later scoped APIs, not in the first shared interface.

### Why A Trait

The trait lets tests and future subsystems speak to "an allocator" without
hard-coding `LinearAllocator`. That matters once Lune has several memory
providers:

- frame allocators for short-lived scratch data
- pool allocators for fixed-size reuse
- arenas for grouped object lifetimes
- renderer staging allocators

The base interface is deliberately small:

```rust
pub trait Allocator {
    fn allocate(&mut self, layout: MemoryLayout) -> MemoryResult<Allocation>;
    fn stats(&self) -> AllocationStats;
}
```

`&mut self` makes mutation explicit. This is simpler than supporting shared
allocator handles or interior mutability before there is a real multi-owner use
case. Later APIs can add scoped frame tokens or handle validation if a subsystem
needs memory to be accessed after allocation.

Bulk reset is split into a second trait:

```rust
pub trait ResettableAllocator: Allocator {
    fn reset(&mut self);
}
```

That keeps pool-style allocators from being forced into a reset contract when
their natural API is allocate/free/reuse.

### Why Not `std::alloc::Allocator`

Rust's standard allocation APIs are useful references, but Lune is not ready to
adopt their shape. `std::alloc::Allocator` is designed around raw memory blocks
and `NonNull<[u8]>` results. That is the right level for collection internals,
but it would force Lune to answer pointer lifetime and deallocation questions
before the metadata-only allocator behavior is complete.

Lune's `Allocator` is an engine learning interface, not a replacement for Rust's
standard allocation traits. It can evolve toward pointer-backed APIs once the
unsafe boundary is smaller and better tested.

### Failure Behavior

Failure is part of the interface. `allocate` returns `MemoryResult<Allocation>`
instead of panicking:

- invalid layouts return `MemoryError::InvalidLayout` before allocation
- capacity failures return `MemoryError::OutOfMemory`
- arithmetic overflow while aligning or sizing an allocation is treated as
  out-of-memory
- failed allocations are counted in allocator stats
- failed allocations must not advance the allocator cursor or used byte count

This keeps engine subsystems honest. A renderer upload path or ECS storage path
can surface allocation failure as a typed engine error instead of relying on
debug assertions or process aborts.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Only inherent methods | Smallest code; easy to read | Future subsystems cannot be generic over allocator type | Single allocator experiments |
| Custom metadata trait | Matches Lune's current learning boundary | Not compatible with Rust collection allocation APIs | Lune M2 |
| `std::alloc::Allocator` shape | Familiar to custom collection work; pointer-backed | Pulls raw pointer and deallocation contracts forward | Later collection internals |
| `GlobalAlloc` | Integrates at process/global allocator boundary | Too broad; unsafe global behavior; no per-subsystem stats | Program-wide allocator replacement |
| Raw pointer API | Direct path to typed memory | Unsafe contract is large and easy to misuse | Later low-level implementation internals |
| Scoped arena API | Can make reset invalidation a lifetime rule | More API design needed; less general for pools | Future frame allocator work |

## Industry Examples

Rust's standard library separates global allocation (`GlobalAlloc`) from
allocator-backed collection internals (`Allocator`). The exact stable surface
changes over time, but the design pressure is visible: allocation APIs must make
layout, ownership, and failure behavior explicit.

Arena allocators such as `bumpalo` expose a scoped allocation model where many
values share one lifetime and are freed together. That is close to Lune's future
frame allocator direction, but Lune keeps the first trait metadata-only so reset
and pointer invalidation are not hidden behind unsafe shortcuts.

Game engines commonly hide subsystem-specific allocators behind local interfaces.
The public details vary, but the pattern is consistent: renderer, asset, and ECS
code should depend on a small memory contract rather than each owning raw
allocation policy.

## Lune Architecture Impact

`lune_memory` remains a base crate. The `Allocator` trait can be used by future
base and runtime crates without introducing dependencies on renderer, ECS, asset,
or platform code.

The current API surface is:

- `MemoryLayout`: validates allocation requests
- `Allocation`: reports offset/size/alignment metadata
- `MemoryError`: reports invalid layouts and out-of-memory failures
- `AllocationStats`: exposes capacity, used bytes, peak usage, and counts
- `Allocator`: shared allocation and stats behavior
- `ResettableAllocator`: bulk-reset behavior for linear/frame-style allocators
- `LinearAllocator`: first concrete implementer

This keeps M2.2 focused on contracts. Real pointer access remains deferred until
Lune has a scoped API or another validation strategy for reset invalidation.

## Decision

Add `lune_memory::Allocator` as a safe, byte-oriented, metadata-returning trait.
It requires `allocate` and `stats`, and provides `capacity_bytes` as a
convenience derived from stats. Add `ResettableAllocator` for allocators that can
invalidate all previous allocations through `reset`. Implement both traits for
`LinearAllocator` so the existing allocator can be exercised through inherent
methods, base trait dispatch, and resettable trait dispatch.

Do not expose raw pointers from the trait in M2.2.

## Contract Tests

The tests live in `crates/lune_memory/tests/allocator_interface.rs`.

They cover:

- `LinearAllocator` can be used through `dyn Allocator`
- `LinearAllocator` can be used through `dyn ResettableAllocator`
- trait allocation returns offset, size, and alignment metadata
- trait reset clears used bytes
- typed out-of-memory behavior is preserved through generic allocator code
- failed allocations update failure stats

## Implementation Handles

- Key types: `Allocator`, `ResettableAllocator`, `MemoryLayout`, `Allocation`,
  `MemoryResult`, `MemoryError`, `AllocationStats`.
- Key invariant: the trait does not promise pointer validity; it promises
  metadata describing allocator-owned storage.
- Error cases: invalid layouts and out-of-memory.
- Edge cases: dynamic dispatch, generic dispatch, failure stats through trait
  calls, reset through resettable trait calls.
- Useful assertions: `dyn Allocator`, `dyn ResettableAllocator`, and
  `impl Allocator` should observe the same behavior as direct `LinearAllocator`
  calls for the methods those traits expose.

## Verification

Run the allocator interface tests directly while working on trait behavior:

```bash
cargo test -p lune_memory --test allocator_interface
```

Run the full local verification command before closing the topic:

```bash
just check
```

## Follow-Up Topics

- M2.3 linear allocator implementation review
- M2.4 frame allocator wrapper
- M2.5 pool allocator
- M2.7 fixed-capacity collection APIs

## Additional Reading

- Rust `std::alloc` module: https://doc.rust-lang.org/stable/std/alloc/index.html
- Rust `GlobalAlloc`: https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html
- Rust `Allocator`: https://doc.rust-lang.org/stable/std/alloc/trait.Allocator.html
- `bumpalo` arena allocator documentation: https://docs.rs/bumpalo/latest/bumpalo/
- `allocator-api2` compatibility crate: https://docs.rs/allocator-api2/latest/allocator_api2/
