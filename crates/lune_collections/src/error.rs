use std::alloc::Layout;
use thiserror::Error;

pub type CollectionResult<T> = std::result::Result<T, CollectionError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CollectionError {
    #[error("fixed vector capacity exceeded: capacity {capacity}")]
    FixedCapacityExceeded { capacity: usize },
    #[error("failed to allocate memory during spillage: layout {layout:?}")]
    AllocationError { layout: Layout },
    #[error("layout overflowed: capacity {capacity}")]
    LayoutOverflow { capacity: isize },
}
