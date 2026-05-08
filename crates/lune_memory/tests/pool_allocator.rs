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
