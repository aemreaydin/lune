use crate::{MemoryError, MemoryLayout, MemoryResult};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_POOL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolAllocation {
    pool_id: u64,
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
    pool_id: u64,

    stats: PoolStats,

    slot_generations: Vec<u64>,
    free_slots: BTreeSet<usize>,
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

        let stats = PoolStats {
            capacity_bytes,
            slot_size: slot_layout.size(),
            slot_align: slot_layout.align(),
            slot_stride,
            slot_count,
            active_slots: 0,
            free_slots: slot_count,
            peak_active_slots: 0,
            successful_allocations: 0,
            failed_allocations: 0,
        };

        Ok(Self {
            slot_layout,
            slot_stride,
            slot_count,
            capacity_bytes,
            pool_id: next_pool_id()?,
            stats,
            slot_generations: vec![0; slot_count],
            free_slots: BTreeSet::from_iter(0..slot_count),
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
        let slot_index = match self.free_slots.first() {
            Some(slot) => *slot,
            None => {
                self.stats.failed_allocations += 1;
                return Err(MemoryError::OutOfMemory {
                    requested_size: self.slot_layout.size(),
                    align: self.slot_layout.align(),
                    capacity: self.capacity_bytes,
                    used: self.capacity_bytes,
                });
            }
        };

        let generation = match self.slot_generations[slot_index].checked_add(1) {
            Some(generation) => generation,
            None => {
                self.stats.failed_allocations += 1;
                return Err(MemoryError::PoolGenerationOverflow { slot_index });
            }
        };
        self.slot_generations[slot_index] = generation;

        let offset = self.slot_stride * slot_index;
        self.free_slots.remove(&slot_index);

        self.stats.active_slots += 1;
        self.stats.free_slots -= 1;
        self.stats.peak_active_slots = self.stats.peak_active_slots.max(self.stats.active_slots);
        self.stats.successful_allocations += 1;

        Ok(PoolAllocation {
            pool_id: self.pool_id,
            slot_index,
            generation,
            offset,
            size: self.slot_layout.size(),
            align: self.slot_layout.align(),
        })
    }

    pub fn free(&mut self, allocation: PoolAllocation) -> MemoryResult<()> {
        if allocation.pool_id != self.pool_id {
            return Err(MemoryError::InvalidPoolAllocation {
                slot_index: allocation.slot_index,
                generation: allocation.generation,
            });
        }

        let Some(&slot_generation) = self.slot_generations.get(allocation.slot_index) else {
            return Err(MemoryError::InvalidPoolAllocation {
                slot_index: allocation.slot_index,
                generation: allocation.generation,
            });
        };

        if self.free_slots.contains(&allocation.slot_index) {
            return Err(MemoryError::DoubleFree {
                slot_index: allocation.slot_index,
                generation: allocation.generation,
            });
        }

        if slot_generation != allocation.generation {
            return Err(MemoryError::DoubleFree {
                slot_index: allocation.slot_index,
                generation: allocation.generation,
            });
        }

        self.free_slots.insert(allocation.slot_index);
        self.stats.active_slots -= 1;
        self.stats.free_slots += 1;

        Ok(())
    }

    pub fn stats(&self) -> PoolStats {
        self.stats
    }
}

fn align_up(value: usize, align: usize) -> Option<usize> {
    let mask = align - 1;
    Some(value.checked_add(mask)? & !mask)
}

fn next_pool_id() -> MemoryResult<u64> {
    NEXT_POOL_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| MemoryError::PoolIdentityExhausted)
}
