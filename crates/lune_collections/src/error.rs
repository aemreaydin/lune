use thiserror::Error;

pub type CollectionResult<T> = std::result::Result<T, CollectionError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CollectionError {
    #[error("fixed vector capacity exceeded: capacity {capacity}")]
    FixedCapacityExceeded { capacity: usize },
}
