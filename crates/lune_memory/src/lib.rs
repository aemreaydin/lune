//! Allocators, arenas, pools, and handle storage for Lune.
//!
//! Production APIs land in the memory learning milestone.

pub mod allocator;
pub mod error;
pub mod frame;
pub mod layout;
pub mod linear;
pub mod pool;
pub mod stats;

pub use allocator::{Allocator, ResettableAllocator};
pub use error::{MemoryError, MemoryResult};
pub use frame::{FrameAllocation, FrameAllocator};
pub use layout::MemoryLayout;
pub use linear::{Allocation, LinearAllocator};
pub use pool::{PoolAllocation, PoolAllocator, PoolStats};
pub use stats::AllocationStats;
