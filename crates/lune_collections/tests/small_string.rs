use std::fmt::Write as _;

use lune_collections::SmallString;

#[test]
fn small_string_starts_empty_with_inline_capacity() {
    let text = SmallString::<8>::new();

    assert_eq!(text.inline_capacity(), 8);
    assert_eq!(text.capacity(), 8);
    assert_eq!(text.len(), 0);
    assert!(text.is_empty());
    assert!(!text.is_spilled());
    assert_eq!(text.as_str(), "");
}

#[test]
fn small_string_uses_inline_storage_until_byte_capacity() {
    let mut text = SmallString::<5>::new();

    text.push_str("lune").unwrap();
    text.push('!').unwrap();

    assert_eq!(text.len(), 5);
    assert_eq!(text.capacity(), 5);
    assert!(!text.is_spilled());
    assert_eq!(text.as_str(), "lune!");
}

#[test]
fn small_string_spills_after_inline_capacity_and_preserves_text() {
    let mut text = SmallString::<4>::new();

    text.push_str("lune").unwrap();
    text.push_str(" engine").unwrap();

    assert_eq!(text.inline_capacity(), 4);
    assert!(text.capacity() >= text.len());
    assert!(text.capacity() > text.inline_capacity());
    assert!(text.is_spilled());
    assert_eq!(text.as_str(), "lune engine");
}

#[test]
fn small_string_preserves_utf8_when_multibyte_crosses_inline_boundary() {
    let mut text = SmallString::<3>::new();

    text.push_str("ab").unwrap();
    text.push('é').unwrap();

    assert!(text.is_spilled());
    assert_eq!(text.len(), "abé".len());
    assert_eq!(text.as_str(), "abé");
    assert!(text.as_str().is_char_boundary(text.len()));
}

#[test]
fn small_string_zero_inline_capacity_spills_on_first_nonempty_push() {
    let mut text = SmallString::<0>::new();

    assert_eq!(text.inline_capacity(), 0);
    assert!(!text.is_spilled());

    text.push_str("a").unwrap();

    assert!(text.is_spilled());
    assert_eq!(text.as_str(), "a");
}

#[test]
fn small_string_empty_push_does_not_force_spill() {
    let mut text = SmallString::<0>::new();

    text.push_str("").unwrap();

    assert_eq!(text.as_str(), "");
    assert!(!text.is_spilled());
}

#[test]
fn small_string_push_appends_unicode_scalar_values() {
    let mut text = SmallString::<8>::new();

    text.push('月').unwrap();
    text.push('🌙').unwrap();

    assert_eq!(text.as_str(), "月🌙");
    assert_eq!(text.len(), "月🌙".len());
}

#[test]
fn small_string_clear_keeps_spilled_capacity_reusable() {
    let mut text = SmallString::<4>::new();
    text.push_str("lune engine").unwrap();
    assert!(text.is_spilled());
    let spilled_capacity = text.capacity();

    text.clear();

    assert_eq!(text.as_str(), "");
    assert!(text.is_empty());
    assert!(text.is_spilled());
    assert_eq!(text.capacity(), spilled_capacity);

    text.push_str("moon").unwrap();
    assert_eq!(text.as_str(), "moon");
}

#[test]
fn small_string_fmt_write_appends_formatted_text() {
    let mut text = SmallString::<4>::new();

    write!(&mut text, "hp {}", 42).unwrap();

    assert_eq!(text.as_str(), "hp 42");
    assert!(text.is_spilled());
}

#[test]
fn small_string_mut_str_allows_in_place_ascii_updates() {
    let mut text = SmallString::<8>::new();

    text.push_str("lune").unwrap();
    text.as_mut_str().make_ascii_uppercase();

    assert_eq!(text.as_str(), "LUNE");
}

#[test]
fn moved_small_string_preserves_initialized_text() {
    let mut text = SmallString::<4>::new();
    text.push_str("lune engine").unwrap();

    let moved = text;

    assert_eq!(moved.as_str(), "lune engine");
}

#[test]
fn moved_inline_small_string_preserves_initialized_text() {
    let mut text = SmallString::<16>::new();
    text.push_str("lune").unwrap();
    assert!(!text.is_spilled());

    let moved = text;

    assert!(!moved.is_spilled());
    assert_eq!(moved.as_str(), "lune");
}
