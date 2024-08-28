use core::alloc::Layout;
use core::fmt;

/// The `TryReserveError` error indicates that there was
/// some kind of failure when trying to grow or shrink some underlying
/// storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TryReserveError {
    /// Error due to the computed capacity exceeding
    /// the collection's maximum (usually [`isize::MAX`] bytes).
    CapacityOverflow,

    /// The memory allocator returned an error.
    AllocError {
        /// The layout of the allocation request that failed.
        layout: Layout,
    },
}

impl fmt::Display for TryReserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("memory allocation failed")?;

        let reason = match self {
            TryReserveError::CapacityOverflow => {
                " because the computed capacity exceeded the collection's maximum"
            }
            TryReserveError::AllocError { .. } => " because the memory allocator returned an error",
        };

        f.write_str(reason)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TryReserveError {}
