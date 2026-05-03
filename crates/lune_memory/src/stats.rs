#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationStats {
    pub(crate) capacity_bytes: usize,
    pub(crate) used_bytes: usize,
    pub(crate) peak_used_bytes: usize,
    pub(crate) successful_allocations: usize,
    pub(crate) failed_allocations: usize,
}

impl Default for AllocationStats {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, 0)
    }
}

impl AllocationStats {
    pub(crate) const fn new(
        capacity_bytes: usize,
        used_bytes: usize,
        peak_used_bytes: usize,
        successful_allocations: usize,
        failed_allocations: usize,
    ) -> Self {
        Self {
            capacity_bytes,
            used_bytes,
            peak_used_bytes,
            successful_allocations,
            failed_allocations,
        }
    }

    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    pub const fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub const fn peak_used_bytes(&self) -> usize {
        self.peak_used_bytes
    }

    pub const fn successful_allocations(&self) -> usize {
        self.successful_allocations
    }

    pub const fn failed_allocations(&self) -> usize {
        self.failed_allocations
    }
}
