use crate::{MemoryError, MemoryResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryLayout {
    size: usize,
    align: usize,
}

impl MemoryLayout {
    pub const fn from_size_align(size: usize, align: usize) -> MemoryResult<Self> {
        if align == 0 || !align.is_power_of_two() {
            return Err(MemoryError::InvalidLayout { size, align });
        }
        Ok(Self { size, align })
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn align(&self) -> usize {
        self.align
    }
}
