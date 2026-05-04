use crate::{Allocation, AllocationStats, MemoryLayout, MemoryResult};

/// Shared interface for byte-oriented Lune allocators.
///
/// This trait is intentionally narrower than `std::alloc::Allocator`. It
/// returns allocation metadata instead of raw pointers while the memory milestone
/// is still establishing lifetime and invalidation rules.
pub trait Allocator {
    fn allocate(&mut self, layout: MemoryLayout) -> MemoryResult<Allocation>;

    fn stats(&self) -> AllocationStats;

    fn capacity_bytes(&self) -> usize {
        self.stats().capacity_bytes()
    }
}

pub trait ResettableAllocator: Allocator {
    fn reset(&mut self);
}
