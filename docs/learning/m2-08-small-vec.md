# M2.8 SmallVec

## Status

Implemented.

## Goal

This topic teaches how an inline-first vector stores common small collections
without heap allocation while still supporting larger cases by spilling to heap
storage. It unlocks a practical middle point between `FixedVec<T, N>` and
ordinary `Vec<T>` for engine lists whose typical size is small but whose worst
case is not fixed.

## Concept

`SmallVec<T, N>` begins with `N` inline slots inside the container itself. While
`len <= N`, pushes initialize inline slots and no heap buffer is needed. When a
push would exceed `N`, the vector allocates heap storage, moves initialized
inline elements into that storage, appends the new value, and switches future
operations to the spilled representation.

The central lesson is representation transition. `FixedVec<T, N>` only needs to
track which inline slots are initialized. `SmallVec<T, N>` must also track where
the initialized elements live:

- inline storage before spill
- heap storage after spill
- length and capacity in either representation

The transition must preserve insertion order, move each initialized value
exactly once, and avoid dropping values from the old inline storage after they
have moved. The container also needs ordinary vector behavior after spill:
pushes grow capacity as needed, pops return the last value, slices expose only
initialized elements, and clearing drops current elements while keeping the
container reusable.

Once a `SmallVec<T, N>` spills, it stays spilled. `pop` and `clear` reduce the
length but do not collapse heap storage back into the inline area. This keeps
later pushes cheap after a temporary spike and matches the rule that
representation transitions only happen in one direction (`Vec::shrink_to_fit`
is an explicit opt-in elsewhere; `SmallVec` does not expose it here).

Moving a `SmallVec<T, N>` is also two different problems depending on state.
After spill, the heap pointer and length move with the container and the inline
slots are no longer authoritative. While inline, the move bit-copies the entire
`[MaybeUninit<T>; N]` array along with `len`; the new owner reads from the
relocated inline area and the old binding must no longer be used. The
implementation must be careful that the move-by-bytes still leaves initialized
slots reachable through the new owner and that no inline slot is read or
dropped through the old binding.

Lune should keep this first version narrow. It does not need all `Vec` APIs. The
learning target is correctness around inline storage, spill, movement, drops,
and slice access.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| Always use `Vec<T>` | Simple and battle-tested | Always uses heap allocation for non-empty storage | General engine code where allocation policy does not matter |
| `FixedVec<T, N>` | No heap allocation and predictable capacity | Cannot represent larger cases | Hard-bounded batches and scratch lists |
| `SmallVec<T, N>` | No heap for common small cases, still handles larger cases | Inline-to-heap transition adds unsafe move/drop complexity | Small typical counts with occasional larger cases |
| `Box<[MaybeUninit<T>; N]>` inline substitute | Keeps stack size small | Still heap-allocates the inline area | Large inline capacities that should not bloat stack frames |
| Reference crate `smallvec` | Mature API and many edge cases handled | Replaces the learning implementation | Study material, or a future non-learning dependency |

## Industry Examples

Small-buffer optimization appears in many performance-oriented codebases. C++
standard-library implementations and engine utility libraries often use inline
storage for strings, vectors, or arrays whose common size is tiny. Public Rust
examples include the `smallvec` crate and related fixed-capacity crates such as
`arrayvec`.

In game engines, this pattern fits data such as a handful of visible lights,
temporary render barriers, child lists for common scene nodes, small event
batches, and ECS scratch lists. The exact engine type names vary, but the
tradeoff is common: avoid allocation in the common case without forcing a hard
maximum.

## Lune Architecture Impact

`SmallVec<T, N>` lives in `lune_collections`, alongside `FixedVec<T, N>`. It
should not depend on renderer, ECS, platform, asset, or showcase crates.

Future users can choose among collection contracts:

- `Vec<T>` when allocation policy is not part of the behavior
- `FixedVec<T, N>` when exceeding capacity is an error
- `SmallVec<T, N>` when the common case is bounded but larger cases are valid

This keeps allocation behavior visible without turning custom containers into a
blanket replacement for the Rust standard library.

## Decision

Add a focused `SmallVec<T, N>` with this initial public contract:

- `new`, `default`, `len`, `is_empty`, `capacity`, and `inline_capacity`
- `is_spilled` to make the representation transition testable
- `push`, `pop`, and `clear`
- `as_slice` and `as_mut_slice`
- `Deref<Target = [T]>` and `DerefMut` so slice methods, iteration, and
  indexing work directly without redundant inherent methods
- `AsRef<[T]>`, `AsMut<[T]>`, and `IntoIterator` for `&SmallVec` and
  `&mut SmallVec` to match the surface of `FixedVec` and the standard library

`capacity()` reports the inline capacity `N` before spill. After spill it
reports the underlying heap `Vec<T>`'s capacity, which may exceed `len` and
which monotonically grows as the heap buffer is reallocated. Tests pin only the
weaker invariants `capacity() >= len()` and `capacity() > inline_capacity()`
after spill so the contract does not couple to `Vec`'s growth strategy.

The first implementation may use `Vec<T>` for spilled storage. It should not add
`smallvec`, `arrayvec`, or another learning-area reference crate as a dependency.

## Failing Tests

The tests live in `crates/lune_collections/tests/small_vec.rs` and should
compile and fail at `todo!()` boundaries until the user implements
`SmallVec<T, N>`. The test set covers four families of behavior:

Inline path:

- `small_vec_starts_empty_with_inline_capacity`
- `small_vec_uses_inline_storage_until_capacity`
- `moved_inline_small_vec_preserves_initialized_elements`
- `small_vec_drops_inline_only_elements_when_dropped`

Spill path:

- `small_vec_spills_after_inline_capacity_and_preserves_order`
- `small_vec_pop_crosses_spill_boundary_in_lifo_order`
- `small_vec_mut_slice_allows_in_place_updates_after_spill`
- `moved_small_vec_preserves_initialized_elements`
- `small_vec_drop_only_drops_initialized_elements_once`
- `small_vec_clear_drops_inline_and_spilled_elements_and_allows_reuse`

State-transition contract (once spilled, stays spilled):

- `small_vec_remains_spilled_after_pop_below_inline_capacity`
- `small_vec_remains_spilled_after_clear`
- `small_vec_capacity_after_spill_exceeds_inline_and_holds_len`

Edge cases and randomized equivalence:

- `small_vec_zero_inline_capacity_spills_on_first_push`
- `small_vec_supports_zero_sized_types`
- `small_vec_push_pop_matches_vec_for_operation_sequences` (proptest, 256
  cases, push-weighted 3:1 over pop, sequences up to 256 operations to
  exercise repeated spill-boundary crossings)

## Implementation Handles

- Key types: `SmallVec<T, const N: usize>` and an internal representation that
  distinguishes inline from spilled storage.
- Key invariants: `len <= capacity`; slices expose only initialized elements;
  inline slots are not read or dropped after their values move to heap storage.
- Error cases: allocation failure during spill or heap growth if exposed through
  the public API.
- Edge cases: zero inline capacity, one-slot inline capacity, pushing exactly
  `N`, pushing `N + 1`, popping after spill, clear after spill, moving a spilled
  vector, zero-sized element types, and `Drop` side effects.
- Useful assertions: slice contents match a reference `Vec`, drop counters match
  initialized elements, capacity never falls below length, and `is_spilled`
  changes only after exceeding inline capacity.

## Verification

Run the collection tests while implementing:

```bash
cargo test -p lune_collections --test small_vec
```

Run the focused crate checks before review:

```bash
cargo test -p lune_collections
cargo clippy -p lune_collections --all-targets -- -D warnings
cargo fmt --check
```

Miri should be used when the toolchain supports it:

```bash
cargo miri test -p lune_collections
```

## Follow-Up Topics

- M2.9 `SmallString<N>` reuses the inline-first idea but adds UTF-8 invariants.
- M2.10 ring buffers reuse fixed storage but change insertion/removal order.
- Later ECS and renderer scratch storage can adopt `SmallVec` where larger cases
  are valid but usually rare.

## Additional Reading

- Rust `MaybeUninit`: https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html
- Rust `Vec`: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html
- `smallvec` crate documentation: https://docs.rs/smallvec/latest/smallvec/
- `arrayvec` crate documentation: https://docs.rs/arrayvec/latest/arrayvec/
