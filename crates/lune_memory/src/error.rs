use thiserror::Error;

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MemoryError {
    #[error("invalid memory layout: size {size} with alignment {align}")]
    InvalidLayout { size: usize, align: usize },

    #[error(
        "out of memory: requested {requested_size} bytes with alignment {align}, capacity {capacity} bytes, used {used} bytes"
    )]
    OutOfMemory {
        requested_size: usize,
        align: usize,
        capacity: usize,
        used: usize,
    },
}
