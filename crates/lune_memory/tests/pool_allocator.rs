use lune_memory::{MemoryError, MemoryLayout, PoolAllocator};

fn layout(size: usize, align: usize) -> MemoryLayout {
    MemoryLayout::from_size_align(size, align).unwrap()
}

#[test]
fn pool_allocator_allocates_fixed_size_slots_until_full() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 3).unwrap();

    let first = allocator.allocate().unwrap();
    let second = allocator.allocate().unwrap();
    let third = allocator.allocate().unwrap();

    assert_eq!(first.slot_index(), 0);
    assert_eq!(first.offset(), 0);
    assert_eq!(first.size(), 8);
    assert_eq!(first.align(), 8);

    assert_eq!(second.slot_index(), 1);
    assert_eq!(second.offset(), 8);

    assert_eq!(third.slot_index(), 2);
    assert_eq!(third.offset(), 16);

    let stats = allocator.stats();
    assert_eq!(stats.capacity_bytes(), 24);
    assert_eq!(stats.slot_count(), 3);
    assert_eq!(stats.active_slots(), 3);
    assert_eq!(stats.free_slots(), 0);
    assert_eq!(stats.peak_active_slots(), 3);
    assert_eq!(stats.successful_allocations(), 3);
    assert_eq!(stats.failed_allocations(), 0);
}

#[test]
fn pool_allocator_reports_free_slots_before_pool_is_full() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 3).unwrap();

    let initial_stats = allocator.stats();
    assert_eq!(initial_stats.active_slots(), 0);
    assert_eq!(initial_stats.free_slots(), 3);
    assert_eq!(
        initial_stats.active_slots() + initial_stats.free_slots(),
        initial_stats.slot_count()
    );

    allocator.allocate().unwrap();

    let stats_after_one_allocation = allocator.stats();
    assert_eq!(stats_after_one_allocation.active_slots(), 1);
    assert_eq!(stats_after_one_allocation.free_slots(), 2);
    assert_eq!(
        stats_after_one_allocation.active_slots() + stats_after_one_allocation.free_slots(),
        stats_after_one_allocation.slot_count()
    );

    allocator.allocate().unwrap();

    let stats_after_two_allocations = allocator.stats();
    assert_eq!(stats_after_two_allocations.active_slots(), 2);
    assert_eq!(stats_after_two_allocations.free_slots(), 1);
    assert_eq!(
        stats_after_two_allocations.active_slots() + stats_after_two_allocations.free_slots(),
        stats_after_two_allocations.slot_count()
    );
}

#[test]
fn pool_allocator_reports_out_of_memory_when_no_slots_are_free() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 1).unwrap();

    allocator.allocate().unwrap();

    let err = allocator.allocate().unwrap_err();
    assert_eq!(
        err,
        MemoryError::OutOfMemory {
            requested_size: 8,
            align: 8,
            capacity: 8,
            used: 8,
        },
    );

    let stats = allocator.stats();
    assert_eq!(stats.active_slots(), 1);
    assert_eq!(stats.free_slots(), 0);
    assert_eq!(stats.successful_allocations(), 1);
    assert_eq!(stats.failed_allocations(), 1);
}

#[test]
fn full_pool_reports_reserved_capacity_for_padded_slots() {
    let mut allocator = PoolAllocator::with_layout(layout(6, 4), 3).unwrap();

    allocator.allocate().unwrap();
    allocator.allocate().unwrap();
    allocator.allocate().unwrap();

    let err = allocator.allocate().unwrap_err();
    assert_eq!(
        err,
        MemoryError::OutOfMemory {
            requested_size: 6,
            align: 4,
            capacity: 24,
            used: 24,
        },
    );
}

#[test]
fn freeing_a_slot_makes_it_available_for_reuse() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 2).unwrap();

    let first = allocator.allocate().unwrap();
    let second = allocator.allocate().unwrap();
    allocator.free(first).unwrap();

    let reused = allocator.allocate().unwrap();

    assert_eq!(reused.slot_index(), first.slot_index());
    assert_eq!(reused.offset(), first.offset());
    assert!(reused.generation() > first.generation());
    assert_eq!(second.slot_index(), 1);

    let stats = allocator.stats();
    assert_eq!(stats.active_slots(), 2);
    assert_eq!(stats.free_slots(), 0);
    assert_eq!(stats.peak_active_slots(), 2);
    assert_eq!(stats.successful_allocations(), 3);
}

#[test]
fn freeing_still_live_slot_succeeds_after_another_slot_is_freed() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 3).unwrap();

    let first = allocator.allocate().unwrap();
    let second = allocator.allocate().unwrap();
    let third = allocator.allocate().unwrap();

    allocator.free(first).unwrap();
    allocator.free(second).unwrap();

    let stats = allocator.stats();
    assert_eq!(stats.active_slots(), 1);
    assert_eq!(stats.free_slots(), 2);

    assert_eq!(third.slot_index(), 2);
}

#[test]
fn freeing_allocation_from_another_pool_is_rejected_without_changing_stats() {
    let mut source = PoolAllocator::with_layout(layout(8, 8), 1).unwrap();
    let mut target = PoolAllocator::with_layout(layout(8, 8), 1).unwrap();

    let foreign_allocation = source.allocate().unwrap();

    let err = target.free(foreign_allocation).unwrap_err();
    assert_eq!(
        err,
        MemoryError::InvalidPoolAllocation {
            slot_index: foreign_allocation.slot_index(),
            generation: foreign_allocation.generation(),
        },
    );

    let stats = target.stats();
    assert_eq!(stats.active_slots(), 0);
    assert_eq!(stats.free_slots(), 1);
}

#[test]
fn freeing_stale_handle_after_slot_reuse_reports_stale_allocation() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 1).unwrap();

    let original = allocator.allocate().unwrap();
    allocator.free(original).unwrap();
    let reused = allocator.allocate().unwrap();

    let err = allocator.free(original).unwrap_err();
    assert_eq!(
        err,
        MemoryError::StaleAllocation {
            slot_index: original.slot_index(),
            expected_generation: reused.generation(),
            actual_generation: original.generation(),
        },
    );

    let stats = allocator.stats();
    assert_eq!(stats.active_slots(), 1);
    assert_eq!(stats.free_slots(), 0);
}

#[test]
fn double_free_reports_typed_error_without_changing_stats() {
    let mut allocator = PoolAllocator::with_layout(layout(8, 8), 1).unwrap();

    let allocation = allocator.allocate().unwrap();
    allocator.free(allocation).unwrap();

    let err = allocator.free(allocation).unwrap_err();
    assert_eq!(
        err,
        MemoryError::DoubleFree {
            slot_index: allocation.slot_index(),
            generation: allocation.generation(),
        },
    );

    let stats = allocator.stats();
    assert_eq!(stats.active_slots(), 0);
    assert_eq!(stats.free_slots(), 1);
}

#[test]
fn slot_stride_preserves_alignment_for_each_slot() {
    let mut allocator = PoolAllocator::with_layout(layout(6, 4), 3).unwrap();

    let first = allocator.allocate().unwrap();
    let second = allocator.allocate().unwrap();
    let third = allocator.allocate().unwrap();

    assert_eq!(first.offset(), 0);
    assert_eq!(second.offset(), 8);
    assert_eq!(third.offset(), 16);

    assert_eq!(first.offset() % first.align(), 0);
    assert_eq!(second.offset() % second.align(), 0);
    assert_eq!(third.offset() % third.align(), 0);

    let stats = allocator.stats();
    assert_eq!(stats.slot_size(), 6);
    assert_eq!(stats.slot_align(), 4);
    assert_eq!(stats.slot_stride(), 8);
    assert_eq!(stats.capacity_bytes(), 24);
}
