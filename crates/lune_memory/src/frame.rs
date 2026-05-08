use crate::{
    Allocation, AllocationStats, Allocator, LinearAllocator, MemoryLayout, MemoryResult,
    ResettableAllocator,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameAllocation {
    allocation: Allocation,
    frame_index: u64,
}

impl FrameAllocation {
    pub const fn offset(&self) -> usize {
        self.allocation.offset()
    }

    pub const fn size(&self) -> usize {
        self.allocation.size()
    }

    pub const fn align(&self) -> usize {
        self.allocation.align()
    }

    pub const fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub const fn allocation(&self) -> Allocation {
        self.allocation
    }
}

#[derive(Debug)]
pub struct FrameAllocator {
    linear: LinearAllocator,
    frame_index: u64,
}

impl FrameAllocator {
    pub const fn with_capacity(capacity_bytes: usize) -> Self {
        Self {
            linear: LinearAllocator::with_capacity(capacity_bytes),
            frame_index: 0,
        }
    }

    pub fn allocate(&mut self, layout: MemoryLayout) -> MemoryResult<FrameAllocation> {
        let allocation = self.linear.allocate(layout)?;
        let frame_index = self.frame_index;

        Ok(FrameAllocation {
            allocation,
            frame_index,
        })
    }

    pub fn reset_frame(&mut self) {
        self.frame_index = self
            .frame_index
            .checked_add(1)
            .expect("frame_index overflow");
        self.linear.reset();
    }

    pub fn is_current(&self, allocation: &FrameAllocation) -> bool {
        self.frame_index == allocation.frame_index
    }

    pub fn stats(&self) -> AllocationStats {
        self.linear.stats()
    }

    pub const fn capacity_bytes(&self) -> usize {
        self.linear.capacity_bytes()
    }

    pub const fn frame_index(&self) -> u64 {
        self.frame_index
    }
}

impl Allocator for FrameAllocator {
    fn allocate(&mut self, layout: MemoryLayout) -> MemoryResult<Allocation> {
        Ok(FrameAllocator::allocate(self, layout)?.allocation)
    }

    fn stats(&self) -> AllocationStats {
        FrameAllocator::stats(self)
    }

    fn capacity_bytes(&self) -> usize {
        FrameAllocator::capacity_bytes(self)
    }
}

impl ResettableAllocator for FrameAllocator {
    fn reset(&mut self) {
        self.reset_frame();
    }
}
