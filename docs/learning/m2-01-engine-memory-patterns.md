# M2.1 Engine Memory Patterns

## Status

Implemented.

## Goal

This topic teaches why engines often use explicit allocation patterns instead of
letting every hot path allocate through general-purpose heap APIs. It unlocks the
first `lune_memory` contracts: allocator layouts, typed allocation failures,
allocation statistics, and a linear allocator that later renderer, ECS, asset,
and frame systems can build on.

## Concept

A game engine does not allocate memory for only one kind of workload. Startup,
asset loading, per-frame rendering, ECS storage, scripting, and tools all have
different lifetime and reuse patterns. A single global heap can serve all of
them, but it hides costs and makes performance harder to reason about.

The important questions for an engine allocation are:

- How long should this memory live?
- Does allocation order match free order?
- Is every allocation the same size?
- Can this memory be reset all at once?
- Does the caller need a stable handle instead of a raw pointer?
- What happens when capacity runs out?
- Which unsafe invariants must the allocator preserve?

Rust already gives Lune safe default ownership tools such as `Vec`, `Box`,
`String`, and borrowed slices. Lune should keep using those where they are the
right abstraction. The memory milestone exists for engine-owned paths where a
more explicit lifetime model is useful: per-frame scratch memory, renderer
uploads, fixed-size pools, and future generational storage.

### Alignment

An allocator must return memory at an address compatible with the requested
alignment. A type with 16-byte alignment cannot safely be placed at an arbitrary
byte offset. Even when an allocator internally tracks offsets into a byte buffer,
the offset math must preserve the same rule:

```text
allocation_start % alignment == 0
allocation_end <= capacity
```

Alignment is normally a power of two. Lune's first layout contract rejects zero
alignment and non-power-of-two alignment before allocation. Size zero is allowed
as a layout because Rust and C-style allocation APIs both have to define some
policy for zero-sized values. For Lune's first linear allocator, a zero-sized
allocation should produce aligned metadata without consuming capacity.

### Linear Allocators

A linear allocator, also called a bump allocator, owns a contiguous buffer and a
cursor. Each allocation aligns the cursor, reserves bytes, and moves the cursor
forward. Individual frees are not supported. `reset` moves the cursor back to the
start and invalidates every allocation handed out before the reset.

This pattern fits memory that has a shared lifetime:

- temporary frame scratch data
- transient renderer staging data
- import/build buffers during one load operation
- short-lived command or event batches

The tradeoff is explicit. Allocation is simple and fast, but callers must not
hold old allocations after reset. Lune should document that invalidation rule
before a frame allocator wrapper exposes it to renderer or gameplay code.

### Frame Allocators

A frame allocator is a policy wrapper around one or more linear allocators. The
usual engine pattern is to allocate freely during a frame and reset at a known
frame boundary. Renderers often keep several frames in flight, so a real frame
allocator may rotate through multiple backing buffers instead of reusing one
buffer immediately.

Lune starts with the simpler rule: allocations are valid until the allocator is
reset. Later renderer work can decide whether swapchain frames or GPU fences
require a multi-buffer frame allocator.

### Pool Allocators

A pool allocator owns fixed-size slots. Allocation takes a free slot, and free
returns that slot to the pool. Pools fit objects with stable size and frequent
reuse:

- component storage pages
- command objects
- asset records
- backend resource wrappers

The core pool risks are different from linear allocation. A pool must prevent or
detect double-free, reject pointers or handles that do not belong to it, preserve
alignment for every slot, and expose enough stats to show active and free slots.

### Generational Handles

Handles are not allocators by themselves, but allocator-like storage often needs
them. A generational handle combines an index with a generation counter. The
index finds a slot; the generation detects whether that slot has been recycled
since the handle was created.

This pattern will appear in later milestones for ECS entities and assets. M2 only
needs to establish the memory lesson: stable-looking references can become stale,
and Lune should prefer explicit validation over silently using recycled slots.

### Allocation Stats

Allocator stats are part of the contract, not debug decoration. Without stats,
capacity failures are hard to tune and frame memory regressions are hard to see.
The first stats are intentionally simple:

- capacity in bytes
- currently used bytes
- peak used bytes
- successful allocation count
- failed allocation count

For a linear allocator, `used_bytes` includes padding inserted to satisfy
alignment because that padding is no longer available for later allocations.

### Unsafe Boundaries

Lune's memory APIs should keep unsafe code behind small, reviewed boundaries.
The public contract can be safe even if the implementation eventually uses raw
pointers internally. The unsafe boundary belongs where the allocator turns bytes
into typed memory or manipulates raw backing storage.

The first invariants to preserve are:

- returned allocation ranges are inside the allocator capacity
- returned starts satisfy the requested alignment
- integer arithmetic cannot wrap while computing aligned starts and ends
- reset invalidates earlier allocations and does not leave safe references alive
- typed allocation helpers must initialize memory before safe references exist
- deallocation or reuse cannot double-drop values

Every unsafe block in `lune_memory` should explain which invariant makes that
block sound. Higher-level crates should consume safe APIs and should not need
their own unsafe allocation logic.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Use `Vec`/`Box` everywhere | Safe, familiar, less custom code | Allocation behavior is less visible; hot paths can churn | Most ordinary Rust data structures |
| Use `std::alloc` directly everywhere | Precise control | Unsafe logic spreads across the engine | Very low-level allocation internals only |
| Linear allocator | Fast allocation; one reset frees all | No individual free; reset invalidates everything | Frame scratch, transient batches |
| Frame allocator wrapper | Matches frame lifetime; easy bulk reuse | Needs clear invalidation and frames-in-flight policy | Renderer and simulation scratch memory |
| Pool allocator | Stable fixed-size reuse; predictable capacity | Only fits one slot size unless generalized | Resource records, fixed object pools |
| Arena with typed objects | Good locality; simple lifetime groups | Drop order and destructors need careful design | Import-time or scene-owned object graphs |
| Generational slot storage | Detects stale handles | More metadata; generation overflow policy needed | Entities, assets, reusable slots |
| Third-party allocator crate | Mature implementation | Replaces the learning target | Reference material, not Lune M2 core |

## Industry Examples

Large C++ engines commonly use arenas, pools, and per-frame scratch allocators to
make allocation cost and lifetime visible. Unreal publicly exposes allocator and
memory-stat concepts, though its exact internal allocator choices are broader
than Lune's first milestone. Unity exposes managed APIs to most users, but its
DOTS and native collections model shows the same concern: allocation lifetime,
ownership, and safety policy have to be explicit in performance-sensitive code.

Rust engine projects often start with standard containers and add custom storage
where the engine model needs it. Bevy's ECS storage and asset handles are not
Lune's implementation target, but they are useful public examples of stable IDs,
tables, and allocation-aware runtime systems. Crates such as `bumpalo`,
`slotmap`, `smallvec`, and `arrayvec` are good references for API ergonomics and
edge cases. ADR-0003 keeps them as references because memory and collection
internals are core learning areas for Lune.

## Lune Architecture Impact

`lune_memory` is a base crate. It should not depend on renderer, ECS, asset,
platform, or showcase crates. Higher-level crates can depend on it when they need
engine-owned allocation policies.

The first implementation target is intentionally narrow:

- `MemoryLayout` validates size/alignment requests.
- `MemoryError` reports invalid layouts and out-of-capacity allocation attempts.
- `Allocation` records offset, size, and alignment metadata.
- `AllocationStats` reports capacity, used bytes, peak bytes, and counts.
- `LinearAllocator` defines the bump allocation and reset contract.

This does not mean all engine code must immediately use custom allocators.
Standard Rust containers remain acceptable until a subsystem has a clear
engine-owned lifetime or capacity requirement. The memory milestone should teach
the invariants first, then introduce use sites deliberately.

## Decision

Lune starts memory work with a safe public allocator contract and an internal
implementation that may use unsafe only where the invariant requires it. The
first tests are offset-based so they can validate alignment, capacity, reset, and
stats behavior before raw pointer APIs are introduced.

The first linear allocator policy is:

- invalid alignment is a typed `MemoryError::InvalidLayout`
- allocation aligns the bump cursor before reserving bytes
- out-of-memory is a typed `MemoryError::OutOfMemory`
- failed allocations do not advance the cursor
- reset sets used bytes back to zero
- peak usage and allocation counters remain visible after reset
- zero-sized allocations are valid and do not consume capacity

Raw pointer access, typed allocation helpers, drop handling, and frame/pool
wrappers are follow-up learning work. Keeping the first contract metadata-based
makes the early safety boundary easier to test.

## Contract Tests

The initial tests live in `crates/lune_memory/tests/allocator_contract.rs`.

They define this behavior:

- invalid zero and non-power-of-two alignments are rejected
- linear allocations start at aligned offsets
- used bytes include alignment padding
- out-of-memory returns a typed error and does not advance used bytes
- alignment arithmetic overflow returns a typed out-of-memory error
- reset reuses capacity and preserves peak stats
- zero-sized allocations do not consume capacity
- stats track successful and failed allocation counts

These tests now pass and define the current metadata-only allocator contract.

## Implementation Handles

- Key types: `MemoryLayout`, `MemoryError`, `Allocation`, `AllocationStats`,
  `LinearAllocator`.
- Key invariants: alignment is nonzero and power-of-two; allocation ranges stay
  inside capacity; reset invalidates previous allocations.
- Error cases: invalid layout and out-of-memory.
- Edge cases: zero-sized allocations, exact-capacity allocations, padding before
  an aligned allocation, failed allocation followed by a successful smaller
  allocation, reset after allocations.
- Useful assertions: failed allocation leaves `used_bytes` unchanged; successful
  allocation updates peak and success count; failed allocation updates failure
  count.

## Verification

Run the memory contract test directly while working on allocator behavior:

```bash
cargo test -p lune_memory --test allocator_contract
```

Run the full local verification command before closing the topic:

```bash
just check
```

Miri should be introduced once allocator internals contain meaningful unsafe or
drop-sensitive code:

```bash
just miri
```

## Follow-Up Topics

- M2.2 allocator interfaces
- M2.3 linear allocator implementation
- M2.4 frame allocator wrapper
- M2.5 pool allocator
- M3.1 generational handles

## Additional Reading

- Rust `std::alloc::Layout`: https://doc.rust-lang.org/stable/std/alloc/struct.Layout.html
- The Rust Book, Unsafe Rust: https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html
- The Rustonomicon, What Unsafe Can Do: https://doc.rust-lang.org/nomicon/what-unsafe-does.html
- `bumpalo` bump allocation arena documentation: https://docs.rs/crate/bumpalo/latest
- Ginger Bill, Memory Allocation Strategies series: https://www.gingerbill.org/series/memory-allocation-strategies/
