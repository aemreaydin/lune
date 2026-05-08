# M2.4 Frame Allocator Wrapper

## Status

Implemented.

## Goal

This topic teaches how Lune turns a linear allocator into a per-frame scratch
allocator with explicit reset and invalidation rules. It unlocks transient
memory for renderer command building, visibility lists, simulation scratch, and
other work that is discarded at a frame boundary.

## Concept

A frame allocator is a policy wrapper around a lower-level allocator. The
underlying allocation strategy can still be linear, but the public meaning
changes:

- allocations belong to the current frame
- `reset_frame` starts a new frame lifetime
- all allocations from older frames are stale after reset
- capacity is reused instead of freed one allocation at a time

The important difference from a raw `LinearAllocator` is the lifetime boundary.
A linear allocator only says "these bytes were reserved until reset." A frame
allocator says "these bytes are frame-local scratch and must not be used after
the frame advances."

For M2.4, Lune keeps the API metadata-only. `FrameAllocation` records the
underlying allocation metadata plus a frame generation:

- `offset`: byte offset in the frame allocator's storage model
- `size`: number of reserved bytes
- `align`: alignment used for the request
- `frame_index`: generation of the frame that produced the allocation

The generation makes reset invalidation visible without introducing raw pointer
or reference lifetimes too early. A stale allocation is one whose
`frame_index()` is not equal to the allocator's current `frame_index()`.

### Reset Invalidation

`reset_frame` is a bulk invalidation operation. It must:

- reset current used bytes to zero
- preserve peak usage and allocation counters
- advance the frame generation
- make old `FrameAllocation` metadata fail `is_current`

Frame generation should advance with checked arithmetic. If a process somehow
exhausts `u64` frame generations, `reset_frame` should panic with a clear message
rather than wrap and make old allocations appear current again. This is not a
recoverable allocation failure; it means the invalidation generation can no
longer represent the lifetime boundary safely.

It does not drop typed values or run destructors in M2.4 because no typed memory
access exists yet. Future pointer-backed APIs must decide where destructors run
and how reset safety is enforced.

### Frames In Flight

Real renderers often keep multiple frames in flight. That means CPU code may
start frame N + 1 while GPU work from frame N is still reading upload buffers or
command data. A production renderer may need one allocator per in-flight frame,
or a ring of frame arenas guarded by fences.

M2.4 deliberately implements a single-frame wrapper first. It teaches the reset
and invalidation contract without pulling renderer synchronization policy into
`lune_memory`.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Single frame wrapper | Simple lifetime boundary; easiest to test | Cannot model frames in flight by itself | First M2 frame scratch API |
| Ring of frame allocators | Matches renderer frames in flight | Requires synchronization/fence policy | Renderer staging after Vulkan bring-up |
| Scoped frame token | Can encode lifetime in Rust types | More API design; harder before raw access exists | Later typed allocation APIs |
| Direct `LinearAllocator` use | Minimal code | Reset meaning is implicit at call sites | Very small internal experiments |

## Industry Examples

Frame allocators and scratch arenas are common in game engines because they make
temporary allocation cheap and predictable. Renderer systems frequently use
per-frame arenas for command lists, upload staging descriptions, transient draw
data, and visibility results.

Modern graphics engines with multiple frames in flight often keep separate
transient memory pools per frame or per command context. Public talks and engine
writeups usually describe the pattern rather than every implementation detail:
bulk reset is safe only when all users of the old frame's memory are done.

Rust arena APIs such as `bumpalo` demonstrate a similar grouped-lifetime idea,
but Lune keeps the first frame wrapper metadata-only so reset invalidation can be
tested before references and destructors are involved.

## Lune Architecture Impact

`FrameAllocator` lives in `lune_memory` and wraps `LinearAllocator`. Higher-level
crates should eventually use it for engine-owned temporary work where the frame
lifetime is explicit.

The current API surface is:

- `FrameAllocator`: owns a linear allocator and current frame generation
- `FrameAllocation`: carries allocation metadata plus `frame_index`
- `reset_frame`: starts a new frame and invalidates older allocations
- `is_current`: checks whether allocation metadata belongs to the current frame
- `Allocator` and `ResettableAllocator`: trait dispatch remains available

The wrapper does not depend on renderer, ECS, platform, or showcase crates. A
future renderer can decide whether it needs one `FrameAllocator`, a ring of
allocators, or a backend-specific staging allocator on top of the same concepts.

## Decision

Add `FrameAllocator` as a single-frame wrapper over `LinearAllocator`. It should
delegate allocation math and stats to the linear allocator, stamp successful
allocations with the current frame generation, and advance that generation on
`reset_frame`.

Expose `FrameAllocation` instead of returning bare `Allocation` from the
inherent frame API. This makes stale-frame misuse observable through tests
without adding raw pointers. Implement the existing allocator traits as a
compatibility surface for generic code, with the caveat that `Allocator`
returns only the base `Allocation` metadata.

Do not solve multi-frame renderer synchronization in M2.4.

## Contract Tests

The tests live in `crates/lune_memory/tests/frame_allocator.rs`.

They verify this behavior:

- frame allocations carry offset, size, alignment, and frame generation
- multiple allocations in one frame update used bytes and stats
- `reset_frame` resets used bytes and reuses capacity
- reset advances `frame_index`
- old frame allocations are no longer current after reset
- reset with no allocations still advances `frame_index`
- reset after a failed allocation reuses capacity without clearing failure stats
- zero-sized frame allocations align metadata without consuming capacity
- out-of-memory returns typed errors and does not advance frame usage
- smaller allocations can succeed after a failed allocation
- `ResettableAllocator::reset` follows frame reset semantics
- `ResettableAllocator` trait dispatch can allocate again after reset
- `Allocator` trait dispatch remains usable for base allocation metadata

These tests now pass against the production implementation.

## Implementation Handles

- Key types: `FrameAllocator`, `FrameAllocation`, `LinearAllocator`,
  `MemoryLayout`, `MemoryError`, `AllocationStats`.
- Key invariants: frame allocations belong to exactly one frame generation;
  reset invalidates older generations; nonzero allocation ranges stay inside
  capacity; failed allocations do not advance usage or frame generation.
- Error cases: capacity exhaustion and arithmetic overflow should surface as
  `MemoryError::OutOfMemory` through the wrapped `LinearAllocator`.
- Overflow policy: advancing `frame_index` should use checked arithmetic and
  panic on `u64` exhaustion instead of wrapping.
- Edge cases: reset after no allocations, reset after failed allocation,
  zero-sized frame allocations, stale allocation checks, trait dispatch after
  reset.
- Useful assertions: `is_current` is true before reset and false after reset;
  `frame_index` increments on each frame reset; stats behavior matches
  `LinearAllocator`.

## Verification

Run the frame allocator tests directly while working on frame reset and
invalidation behavior:

```bash
cargo test -p lune_memory --test frame_allocator
```

Run the related allocator contract and trait tests when changing shared
allocator behavior:

```bash
cargo test -p lune_memory --test allocator_contract
cargo test -p lune_memory --test allocator_interface
cargo test -p lune_memory --test frame_allocator
```

Run the full local verification command before closing the topic:

```bash
just check
```

Miri becomes relevant once frame allocations expose raw or typed memory access:

```bash
just miri
```

## Follow-Up Topics

- M2.5 pool allocator
- M2.7 fixed-capacity collection APIs
- Renderer staging allocator design
- Multi-frame renderer synchronization

## Additional Reading

- Rust `std::alloc::Layout`: https://doc.rust-lang.org/stable/std/alloc/struct.Layout.html
- `bumpalo` arena allocator documentation: https://docs.rs/bumpalo/latest/bumpalo/
- Vulkan tutorial, frames in flight: https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Frames_in_flight
- Vulkan Guide, synchronization examples: https://docs.vulkan.org/guide/latest/synchronization_examples.html
