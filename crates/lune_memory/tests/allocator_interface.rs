use lune_memory::{Allocator, LinearAllocator, MemoryError, MemoryLayout, ResettableAllocator};

fn layout(size: usize, align: usize) -> MemoryLayout {
    MemoryLayout::from_size_align(size, align).unwrap()
}

#[test]
fn linear_allocator_can_be_used_through_allocator_trait_object() {
    let mut linear = LinearAllocator::with_capacity(32);
    let allocator: &mut dyn ResettableAllocator = &mut linear;

    let allocation = allocator.allocate(layout(8, 8)).unwrap();

    assert_eq!(allocation.offset(), 0);
    assert_eq!(allocation.size(), 8);
    assert_eq!(allocation.align(), 8);
    assert_eq!(allocator.capacity_bytes(), 32);
    assert_eq!(allocator.stats().used_bytes(), 8);

    allocator.reset();

    assert_eq!(allocator.stats().used_bytes(), 0);
}

#[test]
fn allocator_trait_preserves_typed_failure_behavior() {
    fn allocate_too_much(allocator: &mut impl Allocator) -> MemoryError {
        allocator.allocate(layout(16, 8)).unwrap_err()
    }

    let mut allocator = LinearAllocator::with_capacity(8);

    let err = allocate_too_much(&mut allocator);

    assert_eq!(
        err,
        MemoryError::OutOfMemory {
            requested_size: 16,
            align: 8,
            capacity: 8,
            used: 0,
        },
    );
    assert_eq!(allocator.stats().failed_allocations(), 1);
}

#[test]
fn linear_allocator_can_be_used_through_base_allocator_trait_object() {
    let mut linear = LinearAllocator::with_capacity(16);
    let allocator: &mut dyn Allocator = &mut linear;

    let allocation = allocator.allocate(layout(4, 4)).unwrap();

    assert_eq!(allocation.offset(), 0);
    assert_eq!(allocator.stats().used_bytes(), 4);
}
