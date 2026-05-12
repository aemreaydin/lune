//! Game-oriented collection types for Lune.
//!
//! Production APIs land in the collections learning milestone.

pub mod error;
pub mod fixed_vec;

pub use error::{CollectionError, CollectionResult};
pub use fixed_vec::FixedVec;
