# M2.10 Ring Buffer

## Status

Implemented.

## Goal

Build a bounded FIFO container for engine event queues, log windows, and later
job queue internals. The point of this milestone is to practice fixed-capacity
storage where insertion and removal happen at different ends and logical order
can wrap around the physical array.

## Concept

A ring buffer keeps three pieces of state:

- `head`: the physical index of the oldest initialized element.
- `len`: how many initialized elements are currently stored.
- `policy`: what to do when `push_back` is called while the buffer is full.

The tail slot is derived from `head + len` modulo capacity. When an item is
popped, `head` moves forward. When an item is pushed, the derived tail moves
forward. Physical indexes wrap, but iteration must always present logical FIFO
order.

The container differs from `FixedVec<T, N>` and `SmallVec<T, N>` in three ways.
First, removal is at the opposite end from insertion. Second, the initialized
slots are a possibly-wrapped contiguous range, not a prefix of the array.
Third, when the buffer is full, the container has a policy — reject the new
value or overwrite the oldest — instead of an unconditional error.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| `VecDeque<T>` | Standard, fully featured, growable | Heap-allocated, always allows growth | General FIFO where allocation policy does not matter |
| `Vec<T>` + `remove(0)` | Familiar | `remove(0)` is O(n) | Tiny queues only |
| `RingBuffer<T, N>` | Fixed capacity, O(1) push/pop, no allocation, explicit overflow policy | Hard maximum; iteration may cross the wrap boundary | Bounded engine queues with a known size budget |
| Reference crate `arraydeque` or `heapless::spsc::Queue` | Mature behavior and APIs | Replaces this learning implementation | Study material or future non-learning dependency |

## Industry Examples

Ring buffers appear throughout systems and engine code. Input subsystems (SDL,
Wayland) use ring buffers for event queues. Audio engines use them to bridge
real-time callbacks with other threads. Loggers commonly keep a fixed-capacity
ring of recent entries for crash dumps. The general shape — a bounded FIFO
with explicit behavior at the boundary — is one of the most reused container
patterns in systems software.

Rust reference crates include `arraydeque` for a fixed-capacity deque and
`heapless::spsc::Queue` for a lock-free single-producer/single-consumer ring.
Lune treats them as references, not dependencies, because wraparound and
overflow policy are the learning target here.

## Lune Architecture Impact

`RingBuffer<T, N>` belongs in `lune_collections`. It should not depend on
renderer, ECS, platform, or asset crates.

Future likely uses include:

- engine event queues (input, window, timer events)
- recent-frame statistics windows for diagnostics
- bounded log buffers for crash dumps
- internal job queue staging buffers

It joins `FixedVec<T, N>`, `SmallVec<T, N>`, and `SmallString<N>` as the
fourth `lune_collections` primitive. Choose `RingBuffer` when removal is at
the opposite end from insertion and the worst case must stay bounded.

## Decision

Add `RingBuffer<T, N>` to `lune_collections` with this initial public contract:

```rust
pub enum RingBufferOverflowPolicy {
    Reject,
    OverwriteOldest,
}

pub struct RingBuffer<T, const N: usize>;
pub struct RingBufferIter<'a, T>;

impl<T, const N: usize> RingBuffer<T, N> {
    pub const fn new(policy: RingBufferOverflowPolicy) -> Self;
    pub const fn rejecting() -> Self;
    pub const fn overwriting_oldest() -> Self;
    pub const fn capacity(&self) -> usize;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn is_full(&self) -> bool;
    pub fn overflow_policy(&self) -> RingBufferOverflowPolicy;
    pub fn push_back(&mut self, value: T) -> CollectionResult<Option<T>>;
    pub fn pop_front(&mut self) -> Option<T>;
    pub fn clear(&mut self);
    pub fn as_slices(&self) -> (&[T], &[T]);
    pub fn iter(&self) -> RingBufferIter<'_, T>;
}

impl<'a, T, const N: usize> IntoIterator for &'a RingBuffer<T, N> {
    type Item = &'a T;
    type IntoIter = RingBufferIter<'a, T>;
}
```

`push_back` returns:

- `Ok(None)` when the value was inserted without overwriting anything.
- `Ok(Some(oldest))` when `OverwriteOldest` replaced the oldest element.
- `Err(CollectionError::RingBufferFull { capacity })` when `Reject` would
  exceed capacity, or when `OverwriteOldest` is used with `N == 0`.

For `N == 0`, the buffer is both empty and full. Both policies return
`RingBufferFull { capacity: 0 }` from `push_back` — there is no slot to
overwrite.

## Invariants

- Only `len` slots are initialized.
- `len <= N` always holds.
- If `N == 0`, no element is ever stored.
- `iter`, `IntoIterator for &RingBuffer`, and `as_slices` expose FIFO order.
- `RingBufferIter::size_hint`, `RingBufferIter::len` (via `ExactSizeIterator`),
  and `RingBuffer::len` must agree at every step.
- `clear` and `Drop` drop exactly the initialized elements.
- Rejecting a value leaves the existing buffer contents unchanged.
- Overwriting returns the removed oldest element to the caller.

## Tests

The tests cover:

- empty state and explicit overflow policy
- reject policy preserving existing contents on overflow
- overwrite policy returning removed values and keeping newest items
- FIFO `pop_front`
- wraparound iteration order
- contiguous and wrapped `as_slices`
- `IntoIterator for &RingBuffer` iteration
- `ExactSizeIterator::len` and `size_hint` decreasing across `next()` calls
- `clear` and `Drop`
- zero-capacity full/empty behavior for both policies
- bounded `VecDeque` equivalence for reject and overwrite policies

## Implementation Handles

- Key types: `RingBuffer<T, const N: usize>` backed by `[MaybeUninit<T>; N]`,
  `head: usize`, `len: usize`, and `policy: RingBufferOverflowPolicy`.
- Key invariants: only the `len` slots starting at `head` (mod `N`) are
  initialized; FIFO order is preserved across wrap; `len <= N`.
- Drop behavior: `Drop` delegates to `clear`, and `clear` drops exactly the
  initialized slots even when the logical contents are wrapped.
- Useful wrap helper:

  ```rust
  const fn wrap(start: usize, offset: usize, cap: usize) -> usize {
      if cap == 0 { 0 } else { (start + offset) % cap }
  }
  ```

- Insertion cases:
  - Not full: write to the slot at `wrap(head + len)`, then increment `len`.
  - Full and `Reject`: return `RingBufferFull` without touching storage.
  - Full and `OverwriteOldest`, `N > 0`: read the old `head` slot, write the
    new value into that slot, advance `head`, and return the old value.
  - Full and `OverwriteOldest`, `N == 0`: return `RingBufferFull { capacity: 0 }`.
- `as_slices`:
  - If `head + len <= N`, first is `head..head + len`, second is empty.
  - Otherwise, first is `head..N`, second is `0..(head + len - N)`.
- Iterator: the simplest implementation stores the two `&[T]` slices returned
  by `as_slices` plus a cursor. `ExactSizeIterator::len` and
  `Iterator::size_hint` must report the same remaining count and decrease in
  lockstep with `next()`.

## Verification

Run the ring buffer tests while implementing:

```bash
cargo test -p lune_collections --test ring_buffer
```

Run the focused crate checks before review:

```bash
cargo test -p lune_collections
cargo clippy -p lune_collections --all-targets -- -D warnings
cargo fmt --check
```

Miri should be used when the toolchain supports it:

```bash
cargo +nightly miri test -p lune_collections --test ring_buffer
```

## Follow-Up Topics

- Job systems and event loops can adopt `RingBuffer` for bounded queues.
- Single-producer / single-consumer lock-free ring buffers extend this
  pattern for threading; see `heapless::spsc::Queue` and `crossbeam::queue`.
- A growable counterpart (resize on overflow) is intentionally out of scope;
  the bounded variant is the learning target.

## Additional Reading

- Rust `VecDeque`: https://doc.rust-lang.org/stable/std/collections/struct.VecDeque.html
- Rust `MaybeUninit`: https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html
- `arraydeque` crate: https://docs.rs/arraydeque/latest/arraydeque/
- `heapless::spsc::Queue`: https://docs.rs/heapless/latest/heapless/spsc/struct.Queue.html
