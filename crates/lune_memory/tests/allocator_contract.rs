use lune_memory::{LinearAllocator, MemoryError, MemoryLayout};

fn layout(size: usize, align: usize) -> MemoryLayout {
    MemoryLayout::from_size_align(size, align).unwrap()
}

#[test]
fn memory_layout_rejects_zero_alignment() {
    let err = MemoryLayout::from_size_align(8, 0).unwrap_err();

    assert_eq!(err, MemoryError::InvalidLayout { size: 8, align: 0 });
}

#[test]
fn memory_layout_rejects_non_power_of_two_alignment() {
    let err = MemoryLayout::from_size_align(8, 3).unwrap_err();
    assert_eq!(err, MemoryError::InvalidLayout { size: 8, align: 3 });

    let err = MemoryLayout::from_size_align(8, 6).unwrap_err();
    assert_eq!(err, MemoryError::InvalidLayout { size: 8, align: 6 });
}

#[test]
fn linear_allocator_places_allocations_at_aligned_offsets() {
    let mut allocator = LinearAllocator::with_capacity(64);

    let first = allocator.allocate(layout(3, 1)).unwrap();
    let second = allocator.allocate(layout(4, 4)).unwrap();
    let third = allocator.allocate(layout(8, 8)).unwrap();

    assert_eq!(first.offset(), 0);
    assert_eq!(first.size(), 3);
    assert_eq!(first.align(), 1);

    assert_eq!(second.offset(), 4);
    assert_eq!(second.size(), 4);
    assert_eq!(second.align(), 4);

    assert_eq!(third.offset(), 8);
    assert_eq!(third.size(), 8);
    assert_eq!(third.align(), 8);

    let stats = allocator.stats();
    assert_eq!(stats.capacity_bytes(), 64);
    assert_eq!(stats.used_bytes(), 16);
    assert_eq!(stats.peak_used_bytes(), 16);
    assert_eq!(stats.successful_allocations(), 3);
    assert_eq!(stats.failed_allocations(), 0);
}

#[test]
fn linear_allocator_reports_out_of_memory_without_advancing_cursor() {
    let mut allocator = LinearAllocator::with_capacity(16);

    allocator.allocate(layout(12, 4)).unwrap();

    let err = allocator.allocate(layout(8, 8)).unwrap_err();
    assert_eq!(
        err,
        MemoryError::OutOfMemory {
            requested_size: 8,
            align: 8,
            capacity: 16,
            used: 12,
        },
    );

    let stats_after_failure = allocator.stats();
    assert_eq!(stats_after_failure.used_bytes(), 12);
    assert_eq!(stats_after_failure.peak_used_bytes(), 12);
    assert_eq!(stats_after_failure.successful_allocations(), 1);
    assert_eq!(stats_after_failure.failed_allocations(), 1);

    let smaller = allocator.allocate(layout(4, 4)).unwrap();
    assert_eq!(smaller.offset(), 12);
    assert_eq!(allocator.stats().used_bytes(), 16);
}

#[test]
fn linear_allocator_reset_reuses_capacity_and_preserves_peak_stats() {
    let mut allocator = LinearAllocator::with_capacity(32);

    allocator.allocate(layout(5, 1)).unwrap();
    allocator.allocate(layout(8, 8)).unwrap();
    assert_eq!(allocator.stats().used_bytes(), 16);

    allocator.reset();

    let after_reset = allocator.stats();
    assert_eq!(after_reset.capacity_bytes(), 32);
    assert_eq!(after_reset.used_bytes(), 0);
    assert_eq!(after_reset.peak_used_bytes(), 16);
    assert_eq!(after_reset.successful_allocations(), 2);

    let reused = allocator.allocate(layout(16, 16)).unwrap();
    assert_eq!(reused.offset(), 0);
    assert_eq!(allocator.stats().used_bytes(), 16);
}

#[test]
fn linear_allocator_allows_zero_sized_allocations_without_consuming_capacity() {
    let mut allocator = LinearAllocator::with_capacity(8);

    let zero = allocator.allocate(layout(0, 8)).unwrap();

    assert_eq!(zero.offset(), 0);
    assert_eq!(zero.size(), 0);
    assert_eq!(zero.align(), 8);
    assert_eq!(allocator.stats().used_bytes(), 0);
    assert_eq!(allocator.stats().successful_allocations(), 1);
}

#[test]
fn linear_allocator_reports_out_of_memory_when_alignment_math_overflows() {
    let mut allocator = LinearAllocator::with_capacity(usize::MAX);

    allocator.allocate(layout(usize::MAX - 1, 1)).unwrap();

    let err = allocator.allocate(layout(1, 8)).unwrap_err();
    assert_eq!(
        err,
        MemoryError::OutOfMemory {
            requested_size: 1,
            align: 8,
            capacity: usize::MAX,
            used: usize::MAX - 1,
        },
    );

    let stats = allocator.stats();
    assert_eq!(stats.used_bytes(), usize::MAX - 1);
    assert_eq!(stats.peak_used_bytes(), usize::MAX - 1);
    assert_eq!(stats.successful_allocations(), 1);
    assert_eq!(stats.failed_allocations(), 1);
}
