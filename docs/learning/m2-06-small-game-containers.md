# M2.6 Small Game Containers

## Status

Implemented.

## Goal

This topic teaches why game engines often use small, bounded, or inline
containers instead of routing every short-lived collection through heap-backed
`Vec` and `String` values. It unlocks the `lune_collections` work for
`FixedVec<T, N>`, `SmallVec<T, N>`, `SmallString<N>`, and ring buffers.

## Concept

Rust's standard containers are excellent defaults. `Vec<T>` and `String` are
safe, flexible, well-optimized, and should remain acceptable in ordinary engine
code. Lune's custom collection milestone exists for paths where the allocation
policy is part of the behavior being learned or tuned.

Small game containers optimize for one or more of these properties:

- fixed maximum capacity
- inline storage for common small cases
- no allocation in hot paths
- predictable failure when capacity is exhausted
- stable iteration order for tests and debugging
- explicit drop behavior for manually managed storage

The key learning target is not replacing the Rust standard library everywhere.
It is understanding the contracts that performance-oriented containers need to
make visible: capacity, overflow policy, initialization state, element drops,
UTF-8 validity, and wraparound behavior.

### FixedVec

`FixedVec<T, N>` is a vector-like container with exactly `N` slots of inline
storage. It never spills to the heap. Pushing succeeds while `len < N`; pushing
when full returns an explicit error or the rejected value.

This makes capacity pressure visible. A renderer command batch, event scratch
list, or small component staging list can state its expected maximum and fail
predictably if the caller exceeds it.

The implementation lesson is initialization tracking. The backing storage has
room for `N` values, but only the first `len` values are initialized at any time.
The container must only read, iterate, and drop initialized elements.

### SmallVec

`SmallVec<T, N>` starts like `FixedVec<T, N>` by storing up to `N` values inline.
When more values are needed, it spills into heap storage and behaves more like
`Vec<T>`.

The point is optimizing the common case without forbidding larger cases. Engine
data often has a small typical count and a larger worst case:

- visible lights affecting one object
- child entities for common scene nodes
- temporary render passes or barriers
- small lists of changed components or resources

The difficult part is the transition between inline and spilled storage. Moving
from inline storage to heap storage must preserve order, not double-drop moved
values, and keep capacity/length correct. Moving the container itself must also
keep its initialized elements valid.

### SmallString

`SmallString<N>` applies the same inline-first idea to UTF-8 text. It is useful
for names, labels, debug markers, short paths, log scopes, and other strings
that are usually small but not always.

The central invariant is stronger than a byte buffer: every public string view
must be valid UTF-8. Appending text can fail or spill, but it must not leave the
container with invalid partial bytes. Boundary cases around multi-byte Unicode
matter even if most engine labels are ASCII.

Lune should keep the initial API small. Formatting integration and advanced
string operations can come after construction, push, clear, conversion, and
borrowed string access are correct.

### Ring Buffer

A ring buffer stores elements in a fixed-capacity circular array. It is useful
when producers and consumers advance through data in FIFO order:

- recent diagnostics messages
- input events
- fixed-size telemetry samples
- future job queues or command queues

The storage wraps around instead of shifting elements. The main policy decision
is what happens when the buffer is full:

- reject the new element and keep existing data
- overwrite the oldest element
- expose separate APIs for both policies

For Lune's first implementation, the policy should be explicit in the API and
tests. Silent overwrite is convenient for logs but surprising for gameplay event
queues. Rejection is safer as a default unless the type name or method name says
otherwise.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Standard `Vec` and `String` | Safe, flexible, familiar, highly optimized | Heap allocation policy is implicit | Most non-hot-path engine code |
| `FixedVec<T, N>` | No heap allocation, explicit capacity, predictable failure | Hard maximum; implementation must manage initialized elements | Bounded hot-path batches and scratch lists |
| `SmallVec<T, N>` | Fast common small path with large-case escape hatch | Inline/spill transitions add complexity | Data with small typical counts and occasional large counts |
| `SmallString<N>` | Avoids heap allocation for short labels and names | Must preserve UTF-8 and handle spill/failure boundaries | Debug names, labels, short identifiers |
| Ring buffer | Efficient FIFO without shifting | Full-buffer policy must be explicit | Events, logs, telemetry, queue-like data |
| Reference crates | Mature APIs and edge-case behavior | Replaces the learning implementation if used directly | Study material and later non-learning dependencies |

## Industry Examples

Inline and bounded containers are common in C++ game engines because many engine
systems know their expected small sizes. Unreal exposes types such as `TArray`
and inline allocator options. Many internal engines have fixed arrays, stack
vectors, ring queues, and custom string types for hot paths and tooling labels.

Rust has mature reference crates that model the same ideas:

- `arrayvec` provides fixed-capacity array-backed vectors and strings.
- `smallvec` provides inline storage with heap spillover.
- `heapless` provides no-allocator collections useful in embedded and bounded
  runtime contexts.
- `arraydeque` and similar crates show ring-buffer/deque tradeoffs.

ADR-0003 keeps these crates as references for this milestone. Lune implements
small containers directly because initialization, drop, capacity, and spillover
behavior are core learning topics.

## Lune Architecture Impact

The custom containers live in `lune_collections`, a base crate. It should not
depend on ECS, renderer, assets, platform, or showcase crates.

Future engine use cases include:

- ECS scratch lists and query internals
- renderer resource transition batches
- platform/input event buffering
- diagnostics ring buffers
- asset and scene import temporary lists
- short debug labels and object names

These containers should remain small, tested, and explicit. They are not a new
standard library for Lune. Higher-level crates should choose them only when the
capacity or allocation policy matters.

## Decision

Introduce `lune_collections` with focused implementations in this order:

1. `FixedVec<T, N>` to learn fixed inline storage and drop invariants.
2. `SmallVec<T, N>` to learn inline-to-heap spillover.
3. `SmallString<N>` to apply inline storage to UTF-8 text.
4. A bounded ring buffer with an explicit full-buffer policy.

Do not add `smallvec`, `arrayvec`, or other reference crates as dependencies for
these learning implementations. Use them only as API and edge-case references.

## Failing Tests

M2.6 is the explanation and test-plan gate. The implementation tasks add tests in
the relevant submilestones.

The test plan should emphasize boundary conditions:

- zero capacity where the API supports it
- one element capacity
- exactly full capacity
- pushing one element past capacity
- clear and reuse after full capacity
- iteration order before and after mutation
- drop counts for initialized elements only
- behavior when `T::drop` has observable side effects
- move behavior for containers with initialized inline storage
- inline-to-spill transitions for `SmallVec`
- spill capacity growth without losing order
- UTF-8 boundaries for `SmallString`, especially multi-byte characters
- string append failure or spill behavior that does not leave partial UTF-8
- ring-buffer wraparound after reads and writes
- ring-buffer full and empty states
- overwrite versus reject policy for ring buffers

Example test names for follow-up milestones:

```rust
#[test]
fn fixed_vec_rejects_push_when_full() {
    todo!("user implementation makes this pass");
}

#[test]
fn small_vec_preserves_order_when_spilling_to_heap() {
    todo!("user implementation makes this pass");
}

#[test]
fn small_string_preserves_utf8_when_append_hits_inline_boundary() {
    todo!("user implementation makes this pass");
}

#[test]
fn ring_buffer_iterates_fifo_order_after_wraparound() {
    todo!("user implementation makes this pass");
}
```

## Implementation Handles

- Key types: `FixedVec<T, const N: usize>`, `SmallVec<T, const N: usize>`,
  `SmallString<const N: usize>`, `RingBuffer<T, const N: usize>`.
- Key invariants: only initialized elements are read or dropped; `len <=
  capacity`; iteration order is stable; public string data is valid UTF-8; ring
  head/tail state distinguishes full from empty.
- Error cases: fixed-capacity overflow, allocation failure on spill if exposed,
  invalid UTF-8 construction, empty pop, full ring push under reject policy.
- Edge cases: zero-sized types, zero capacity, one-slot containers, panic during
  drop, multi-byte UTF-8 at inline boundaries, ring wraparound with full and empty
  states.
- Useful assertions: `len() <= capacity()`, collected iteration matches expected
  order, drop counters match initialized elements, `as_str().is_char_boundary(i)`
  for relevant string boundaries, and ring buffer length remains correct across
  wraparound.

## Verification

Run formatting and documentation checks while working on the docs:

```bash
just fmt-check
```

Run the full local verification command after implementation tests are added:

```bash
just check
```

Miri should be used for the container implementations because manual initialized
storage and drop behavior are easy to get subtly wrong:

```bash
just miri
```

## Follow-Up Topics

- M2.7 `FixedVec<T, N>` tests and implementation
- M2.8 `SmallVec<T, N>` tests and implementation
- M2.9 `SmallString<N>` tests and implementation
- M2.10 ring buffer tests and implementation
- Later ECS and renderer scratch storage that consumes these containers

## Additional Reading

- Rust `MaybeUninit`: https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html
- Rust `Drop`: https://doc.rust-lang.org/stable/std/ops/trait.Drop.html
- `arrayvec` crate documentation: https://docs.rs/arrayvec/latest/arrayvec/
- `smallvec` crate documentation: https://docs.rs/smallvec/latest/smallvec/
- `heapless` crate documentation: https://docs.rs/heapless/latest/heapless/
- Rust UTF-8 string docs: https://doc.rust-lang.org/stable/std/string/struct.String.html
