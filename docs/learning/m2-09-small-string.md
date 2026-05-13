# M2.9 SmallString

## Status

Implemented.

## Goal

This topic teaches how inline-first storage changes when the public data is
text instead of arbitrary elements. It unlocks `SmallString<N>` for short engine
labels, debug names, asset tags, log scopes, and other strings that usually fit
inline but must remain valid UTF-8 after every operation.

## Concept

`SmallString<N>` is the string counterpart to `SmallVec<T, N>`. It starts with
`N` inline bytes inside the container. While appended UTF-8 text fits, no heap
allocation is needed. When a push would exceed the inline byte capacity, the
string spills to heap storage and preserves its current text.

The important difference from a byte vector is the invariant: every public view
must be valid UTF-8. Rust `str` values can only be sliced at character
boundaries, and a multi-byte character must never be partially appended. For
example, if the inline storage has one byte left and the caller pushes `é`, the
implementation must spill first or otherwise keep the old string unchanged. It
must not store only the first byte of `é`.

This first version should stay small. It does not need replacement, insertion,
removal, formatting adapters beyond `fmt::Write`, or lossy byte APIs. The core
lesson is byte capacity, spill behavior, valid UTF-8, and reusable storage.

## Alternatives

| Alternative | Pros | Cons | When It Fits |
| --- | --- | --- | --- |
| `String` | Simple, standard, fully featured | Heap allocation policy is implicit | General text where allocation does not matter |
| `SmallString<N>` | Inline common case with heap escape hatch | Must preserve UTF-8 across spill and mutation | Short labels and names with occasional larger cases |
| Fixed byte string | Predictable no-heap behavior | Hard maximum and awkward UTF-8 boundaries | Protocol fields or hard-bounded debug tags |
| `ArrayString`-style design | No heap allocation and clear capacity | Fails when text exceeds fixed capacity | Hard-limited no-allocation text |
| Reference crate `smallstr` or `arrayvec::ArrayString` | Mature behavior and APIs | Replaces this learning implementation | Study material or future non-learning dependency |

## Industry Examples

Small-buffer strings appear in many C++ engines and standard-library
implementations as small-string optimization. Engine code often stores short
names, profiling labels, debug markers, resource keys, and UI text where the
common case is tiny.

Rust reference crates cover similar tradeoffs. `arrayvec::ArrayString` models a
fixed-capacity UTF-8 string, while crates such as `smallstr` provide inline
storage with a larger-case fallback. Lune uses them as references, not
dependencies, because UTF-8 and spill invariants are the learning target here.

## Lune Architecture Impact

`SmallString<N>` belongs in `lune_collections`. Higher-level crates should keep
using `String` unless the allocation policy is part of the behavior being
tested or tuned.

Future likely uses include:

- renderer and diagnostics labels
- short asset and scene names
- profiling scopes
- debug UI labels
- importer scratch strings

This topic also prepares for later asset and scripting work, where runtime text
crosses subsystem boundaries and should keep clear ownership and validity
rules.

## Decision

Add a focused `SmallString<N>` with this initial public contract:

- `new`, `default`, `len`, `is_empty`, `capacity`, and `inline_capacity`
- `is_spilled` to expose the inline-to-heap transition for tests
- `push_str`, `push`, and `clear`
- `as_str` and `as_mut_str`
- `fmt::Write` so formatting can append text through the same invariant checks
- `Deref<Target = str>`, `DerefMut`, `AsRef<str>`, and `AsMut<str>`

`SmallString<N>` should spill rather than reject larger valid UTF-8 text. Once
spilled, it stays spilled across `clear` and any future shrinking operations,
matching `SmallVec<T, N>`.

The first implementation may use `String` for spilled storage. It should not add
`smallstr`, `arrayvec`, or another learning-area reference crate as a dependency.

## Failing Tests

The tests should compile and fail at `todo!()` boundaries until the user
implements `SmallString<N>`.

Inline and spill behavior:

- `small_string_starts_empty_with_inline_capacity`
- `small_string_uses_inline_storage_until_byte_capacity`
- `small_string_spills_after_inline_capacity_and_preserves_text`
- `small_string_zero_inline_capacity_spills_on_first_nonempty_push`
- `small_string_empty_push_does_not_force_spill`

UTF-8 boundaries:

- `small_string_preserves_utf8_when_multibyte_crosses_inline_boundary`
- `small_string_push_appends_unicode_scalar_values`

Mutation, formatting, and movement:

- `small_string_clear_keeps_spilled_capacity_reusable`
- `small_string_fmt_write_appends_formatted_text`
- `small_string_mut_str_allows_in_place_ascii_updates`
- `moved_small_string_preserves_initialized_text`
- `moved_inline_small_string_preserves_initialized_text`

## Implementation Handles

- Key types: `SmallString<const N: usize>` and an internal representation that
  distinguishes inline bytes from spilled string storage.
- Key invariants: `len <= capacity`; every `as_str` view is valid UTF-8; public
  mutation cannot create invalid UTF-8; spilled strings stay spilled; if the
  internal representation owns raw heap bytes, `Drop` must free them exactly
  once and must not drop inline bytes (using `String` for spilled storage makes
  this automatic).
- Error cases: only allocation failure during spill or heap growth. UTF-8
  validity is guaranteed by the `&str` and `char` input types of `push_str` and
  `push`, so there is no UTF-8 error variant to surface.
- Edge cases: zero inline capacity, empty append, exact inline capacity,
  multi-byte characters at the inline boundary, clear after spill, moving inline
  storage, moving spilled storage, and formatting text through `fmt::Write`.
- Useful assertions: `as_str()` equals expected text, `len()` equals
  `as_str().len()`, `as_str().is_char_boundary(len())`, capacity never falls
  below length, and `is_spilled` changes only after exceeding inline capacity
  with non-empty text.

## Verification

Run the string tests while implementing:

```bash
cargo test -p lune_collections --test small_string
```

Run the focused crate checks before review:

```bash
cargo test -p lune_collections
cargo clippy -p lune_collections --all-targets -- -D warnings
cargo fmt --check
```

Miri should be used when the toolchain supports it:

```bash
cargo +nightly miri test -p lune_collections --test small_string
```

## Follow-Up Topics

- M2.10 ring buffers reuse bounded storage but shift the problem to wraparound
  ordering and full-buffer policy.
- Later diagnostics, assets, and debug tooling can choose `SmallString` where
  short labels dominate and allocation behavior matters.

## Additional Reading

- Rust `String`: https://doc.rust-lang.org/stable/std/string/struct.String.html
- Rust `str`: https://doc.rust-lang.org/stable/std/primitive.str.html
- Rust `char`: https://doc.rust-lang.org/stable/std/primitive.char.html
- `arrayvec::ArrayString`: https://docs.rs/arrayvec/latest/arrayvec/struct.ArrayString.html
