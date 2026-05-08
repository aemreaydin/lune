use lune_memory::{Allocator, FrameAllocator, MemoryError, MemoryLayout, ResettableAllocator};

fn layout(size: usize, align: usize) -> MemoryLayout {
    MemoryLayout::from_size_align(size, align).unwrap()
}

#[test]
fn frame_allocator_allocates_in_the_current_frame() {
    let mut allocator = FrameAllocator::with_capacity(32);

    let first = allocator.allocate(layout(8, 8)).unwrap();
    let second = allocator.allocate(layout(4, 4)).unwrap();

    assert_eq!(first.offset(), 0);
    assert_eq!(first.size(), 8);
    assert_eq!(first.align(), 8);
    assert_eq!(first.frame_index(), 0);
    assert!(allocator.is_current(&first));

    assert_eq!(second.offset(), 8);
    assert_eq!(second.size(), 4);
    assert_eq!(second.align(), 4);
    assert_eq!(second.frame_index(), 0);
    assert!(allocator.is_current(&second));

    let stats = allocator.stats();
    assert_eq!(stats.capacity_bytes(), 32);
    assert_eq!(stats.used_bytes(), 12);
    assert_eq!(stats.peak_used_bytes(), 12);
    assert_eq!(stats.successful_allocations(), 2);
    assert_eq!(stats.failed_allocations(), 0);
}

#[test]
fn reset_frame_reuses_capacity_and_invalidates_prior_allocations() {
    let mut allocator = FrameAllocator::with_capacity(16);

    let stale = allocator.allocate(layout(8, 8)).unwrap();

    allocator.reset_frame();

    assert_eq!(allocator.frame_index(), 1);
    assert!(!allocator.is_current(&stale));
    assert_eq!(allocator.stats().used_bytes(), 0);
    assert_eq!(allocator.stats().peak_used_bytes(), 8);

    let current = allocator.allocate(layout(16, 16)).unwrap();

    assert_eq!(current.offset(), 0);
    assert_eq!(current.size(), 16);
    assert_eq!(current.frame_index(), 1);
    assert!(allocator.is_current(&current));
    assert_eq!(allocator.stats().used_bytes(), 16);
    assert_eq!(allocator.stats().successful_allocations(), 2);
}

#[test]
fn reset_frame_without_allocations_advances_frame_and_keeps_stats_empty() {
    let mut allocator = FrameAllocator::with_capacity(16);

    allocator.reset_frame();

    assert_eq!(allocator.frame_index(), 1);
    assert_eq!(allocator.stats().capacity_bytes(), 16);
    assert_eq!(allocator.stats().used_bytes(), 0);
    assert_eq!(allocator.stats().peak_used_bytes(), 0);
    assert_eq!(allocator.stats().successful_allocations(), 0);
    assert_eq!(allocator.stats().failed_allocations(), 0);
}

#[test]
fn reset_frame_after_failed_allocation_reuses_capacity() {
    let mut allocator = FrameAllocator::with_capacity(16);

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

    allocator.reset_frame();

    assert_eq!(allocator.frame_index(), 1);
    assert_eq!(allocator.stats().used_bytes(), 0);
    assert_eq!(allocator.stats().peak_used_bytes(), 12);
    assert_eq!(allocator.stats().successful_allocations(), 1);
    assert_eq!(allocator.stats().failed_allocations(), 1);

    let reused = allocator.allocate(layout(16, 16)).unwrap();
    assert_eq!(reused.offset(), 0);
    assert_eq!(reused.frame_index(), 1);
    assert_eq!(allocator.stats().used_bytes(), 16);
}

#[test]
fn frame_allocator_reports_out_of_memory_without_advancing_frame_usage() {
    let mut allocator = FrameAllocator::with_capacity(16);

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

    assert_eq!(allocator.frame_index(), 0);
    assert_eq!(allocator.stats().used_bytes(), 12);
    assert_eq!(allocator.stats().failed_allocations(), 1);

    let smaller = allocator.allocate(layout(4, 4)).unwrap();
    assert_eq!(smaller.offset(), 12);
    assert_eq!(smaller.frame_index(), 0);
    assert_eq!(allocator.stats().used_bytes(), 16);
}

#[test]
fn zero_sized_frame_allocations_align_metadata_without_advancing_usage() {
    let mut allocator = FrameAllocator::with_capacity(16);

    allocator.allocate(layout(1, 1)).unwrap();

    let zero = allocator.allocate(layout(0, 8)).unwrap();

    assert_eq!(zero.offset(), 8);
    assert_eq!(zero.size(), 0);
    assert_eq!(zero.align(), 8);
    assert_eq!(zero.frame_index(), 0);
    assert!(allocator.is_current(&zero));

    let stats_after_zero = allocator.stats();
    assert_eq!(stats_after_zero.used_bytes(), 1);
    assert_eq!(stats_after_zero.peak_used_bytes(), 1);
    assert_eq!(stats_after_zero.successful_allocations(), 2);

    let next = allocator.allocate(layout(1, 1)).unwrap();
    assert_eq!(next.offset(), 1);
    assert_eq!(next.frame_index(), 0);
    assert_eq!(allocator.stats().used_bytes(), 2);
}

#[test]
fn frame_allocator_reset_is_available_through_resettable_allocator_trait() {
    let mut frame = FrameAllocator::with_capacity(16);
    let allocator: &mut dyn ResettableAllocator = &mut frame;

    let allocation = allocator.allocate(layout(4, 4)).unwrap();
    assert_eq!(allocation.offset(), 0);
    assert_eq!(allocator.stats().used_bytes(), 4);

    allocator.reset();

    assert_eq!(allocator.stats().used_bytes(), 0);
    assert_eq!(frame.frame_index(), 1);
}

#[test]
fn resettable_allocator_trait_allocation_works_after_reset() {
    let mut frame = FrameAllocator::with_capacity(16);
    let allocator: &mut dyn ResettableAllocator = &mut frame;

    allocator.allocate(layout(8, 8)).unwrap();
    allocator.reset();

    let allocation = allocator.allocate(layout(16, 16)).unwrap();

    assert_eq!(allocation.offset(), 0);
    assert_eq!(allocation.size(), 16);
    assert_eq!(allocation.align(), 16);
    assert_eq!(allocator.stats().used_bytes(), 16);
    assert_eq!(allocator.stats().successful_allocations(), 2);
}

#[test]
fn frame_allocator_can_be_used_through_base_allocator_trait() {
    let mut frame = FrameAllocator::with_capacity(8);
    let allocator: &mut dyn Allocator = &mut frame;

    let allocation = allocator.allocate(layout(4, 4)).unwrap();

    assert_eq!(allocation.offset(), 0);
    assert_eq!(allocation.size(), 4);
    assert_eq!(allocation.align(), 4);
    assert_eq!(allocator.stats().used_bytes(), 4);
}
