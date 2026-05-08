use crate::{MemoryError, MemoryLayout, MemoryResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolAllocation {
    slot_index: usize,
    generation: u64,
    offset: usize,
    size: usize,
    align: usize,
}

impl PoolAllocation {
    pub const fn slot_index(&self) -> usize {
        self.slot_index
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolStats {
    capacity_bytes: usize,
    slot_size: usize,
    slot_align: usize,
    slot_stride: usize,
    slot_count: usize,
    active_slots: usize,
    free_slots: usize,
    peak_active_slots: usize,
    successful_allocations: usize,
    failed_allocations: usize,
}

impl PoolStats {
    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    pub const fn slot_size(&self) -> usize {
        self.slot_size
    }

    pub const fn slot_align(&self) -> usize {
        self.slot_align
    }

    pub const fn slot_stride(&self) -> usize {
        self.slot_stride
    }

    pub const fn slot_count(&self) -> usize {
        self.slot_count
    }

    pub const fn active_slots(&self) -> usize {
        self.active_slots
    }

    pub const fn free_slots(&self) -> usize {
        self.free_slots
    }

    pub const fn peak_active_slots(&self) -> usize {
        self.peak_active_slots
    }

    pub const fn successful_allocations(&self) -> usize {
        self.successful_allocations
    }

    pub const fn failed_allocations(&self) -> usize {
        self.failed_allocations
    }
}

#[derive(Debug)]
pub struct PoolAllocator {
    slot_layout: MemoryLayout,
    slot_stride: usize,
    slot_count: usize,
    capacity_bytes: usize,
}

impl PoolAllocator {
    pub fn with_layout(slot_layout: MemoryLayout, slot_count: usize) -> MemoryResult<Self> {
        if slot_layout.size() == 0 {
            return Err(MemoryError::InvalidLayout {
                size: slot_layout.size(),
                align: slot_layout.align(),
            });
        }

        let slot_stride =
            align_up(slot_layout.size(), slot_layout.align()).ok_or(MemoryError::OutOfMemory {
                requested_size: slot_layout.size(),
                align: slot_layout.align(),
                capacity: usize::MAX,
                used: 0,
            })?;
        let capacity_bytes =
            slot_stride
                .checked_mul(slot_count)
                .ok_or(MemoryError::OutOfMemory {
                    requested_size: slot_stride,
                    align: slot_layout.align(),
                    capacity: usize::MAX,
                    used: 0,
                })?;

        Ok(Self {
            slot_layout,
            slot_stride,
            slot_count,
            capacity_bytes,
        })
    }

    pub const fn slot_layout(&self) -> MemoryLayout {
        self.slot_layout
    }

    pub const fn slot_count(&self) -> usize {
        self.slot_count
    }

    pub const fn slot_stride(&self) -> usize {
        self.slot_stride
    }

    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    pub fn allocate(&mut self) -> MemoryResult<PoolAllocation> {
        todo!("implement fixed-size pool slot allocation")
    }

    pub fn free(&mut self, allocation: PoolAllocation) -> MemoryResult<()> {
        let _ = allocation;
        todo!("implement pool slot free and double-free detection")
    }

    pub fn stats(&self) -> PoolStats {
        todo!("report pool active/free slot stats")
    }
}

fn align_up(value: usize, align: usize) -> Option<usize> {
    let mask = align - 1;
    Some(value.checked_add(mask)? & !mask)
}
