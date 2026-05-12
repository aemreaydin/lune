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

    #[error("pool slot layout must have non-zero size (alignment {align})")]
    ZeroSizedSlot { align: usize },

    #[error(
        "pool capacity overflows usize: slot size {slot_size}, alignment {slot_align}, slot count {slot_count}"
    )]
    CapacityOverflow {
        slot_size: usize,
        slot_align: usize,
        slot_count: usize,
    },

    #[error("pool allocation was already freed: slot {slot_index}, generation {generation}")]
    DoubleFree { slot_index: usize, generation: u64 },

    #[error(
        "pool allocation handle is stale: slot {slot_index}, handle generation {actual_generation}, current generation {expected_generation}"
    )]
    StaleAllocation {
        slot_index: usize,
        expected_generation: u64,
        actual_generation: u64,
    },

    #[error(
        "pool allocation does not belong to this allocator: slot {slot_index}, generation {generation}"
    )]
    InvalidPoolAllocation { slot_index: usize, generation: u64 },

    #[error("pool slot generation overflowed: slot {slot_index}")]
    PoolGenerationOverflow { slot_index: usize },

    #[error("pool allocator identity counter exhausted")]
    PoolIdentityExhausted,

    #[error("frame allocator frame index overflowed: current {current_frame_index}")]
    FrameIndexOverflow { current_frame_index: u64 },
}
