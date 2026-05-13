//! Game-oriented collection types for Lune.
//!
//! Production APIs land in the collections learning milestone.

pub mod error;
pub mod fixed_vec;
pub mod small_string;
pub mod small_vec;

pub use error::{CollectionError, CollectionResult};
pub use fixed_vec::FixedVec;
pub use small_string::SmallString;
pub use small_vec::SmallVec;
