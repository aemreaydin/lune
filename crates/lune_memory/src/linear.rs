use crate::{
    AllocationStats, Allocator, MemoryError, MemoryLayout, MemoryResult, ResettableAllocator,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allocation {
    offset: usize,
    size: usize,
    align: usize,
}

impl Allocation {
    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn align(&self) -> usize {
        self.align
    }
}

#[derive(Debug)]
pub struct LinearAllocator {
    capacity_bytes: usize,
    cursor: usize,
    stats: AllocationStats,
}

impl LinearAllocator {
    pub const fn with_capacity(capacity_bytes: usize) -> Self {
        Self {
            capacity_bytes,
            cursor: 0,
            stats: AllocationStats::new(capacity_bytes, 0, 0, 0, 0),
        }
    }

    fn pad_align(&self, alignment: usize) -> usize {
        let modulo = self.cursor & alignment.wrapping_sub(1);
        if modulo != 0 {
            return alignment - modulo;
        }

        0
    }

    pub fn allocate(&mut self, layout: MemoryLayout) -> MemoryResult<Allocation> {
        let pad = self.pad_align(layout.align());
        let aligned_start = match self.cursor.checked_add(pad) {
            Some(aligned_start) => aligned_start,
            None => return self.out_of_memory(layout),
        };
        let alloc_end = match aligned_start.checked_add(layout.size()) {
            Some(alloc_end) => alloc_end,
            None => return self.out_of_memory(layout),
        };

        if alloc_end > self.capacity_bytes {
            return self.out_of_memory(layout);
        }

        let allocation = Allocation {
            offset: aligned_start,
            size: layout.size(),
            align: layout.align(),
        };

        self.stats.successful_allocations += 1;
        if layout.size() != 0 {
            self.stats.used_bytes = alloc_end;
            self.cursor = alloc_end;
        }
        self.stats.peak_used_bytes = self.stats.used_bytes.max(self.stats.peak_used_bytes);

        Ok(allocation)
    }

    fn out_of_memory(&mut self, layout: MemoryLayout) -> MemoryResult<Allocation> {
        self.stats.failed_allocations += 1;
        Err(MemoryError::OutOfMemory {
            requested_size: layout.size(),
            align: layout.align(),
            capacity: self.capacity_bytes,
            used: self.cursor,
        })
    }

    pub fn reset(&mut self) {
        self.stats.used_bytes = 0;
        self.cursor = 0;
    }

    pub fn stats(&self) -> AllocationStats {
        self.stats
    }

    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
}

impl Allocator for LinearAllocator {
    fn allocate(&mut self, layout: MemoryLayout) -> MemoryResult<Allocation> {
        LinearAllocator::allocate(self, layout)
    }

    fn stats(&self) -> AllocationStats {
        LinearAllocator::stats(self)
    }

    fn capacity_bytes(&self) -> usize {
        LinearAllocator::capacity_bytes(self)
    }
}

impl ResettableAllocator for LinearAllocator {
    fn reset(&mut self) {
        LinearAllocator::reset(self);
    }
}
