#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Traits and types for backing memory storage.
pub mod storage;

/// Error types.
pub mod error;

/// Various utilities for this crate.
pub(crate) mod util;
