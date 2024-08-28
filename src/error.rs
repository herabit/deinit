use core::{alloc::Layout, fmt};

/// Error that occurs when failing to reserve additional memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum TryReserveError {
    /// Error that occurs when the computed capacity exceeds the
    /// collection's maximum capacity (usually [`isize::MAX`]).
    #[default]
    CapacityOverflow,

    /// Error that occurs when a memory allocator returns an error.
    AllocError { layout: Layout },
}

impl TryReserveError {
    /// Create a [`TryReserveError`] from the erroneous parameters.
    ///
    /// # Parameters
    ///
    /// - `capacity`: The capacity of the collection.
    /// - `additional`: The amount of additional elements to join the collection.
    /// - `elem_size`: The size of an element.
    /// - `align`: The alignment of an element.
    #[inline]
    pub const fn make(
        capacity: usize,
        additional: usize,
        elem_size: usize,
        align: usize,
    ) -> TryReserveError {
        let Some(size) = capacity.checked_add(additional) else {
            return TryReserveError::CapacityOverflow;
        };

        let Some(size) = size.checked_mul(elem_size) else {
            return TryReserveError::CapacityOverflow;
        };

        let Ok(layout) = Layout::from_size_align(size, align) else {
            return TryReserveError::CapacityOverflow;
        };

        TryReserveError::AllocError { layout }
    }

    /// Create a [`TryReserveError`] from erroneous parameters,
    /// and a compile-time known element type.
    #[inline]
    pub fn new<T>(capacity: usize, additional: usize) -> TryReserveError {
        match capacity.checked_add(additional) {
            Some(new_cap) => match Layout::array::<T>(new_cap) {
                Ok(layout) => TryReserveError::AllocError { layout },
                Err(_) => TryReserveError::CapacityOverflow,
            },
            None => TryReserveError::CapacityOverflow,
        }
    }
}

impl fmt::Display for TryReserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")?;

        f.write_str(match self {
            TryReserveError::CapacityOverflow => {
                " because the computed capacity exceeded the collection's maximum"
            }
            TryReserveError::AllocError { .. } => " because the memory allocator returned an error",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TryReserveError {}
