use core::mem::MaybeUninit;

use crate::Uninit;

/// Trait for initialized data.
pub trait Init {
    /// An uninitialized [`Self`].
    type Uninit: Uninit<Init = Self> + ?Sized;

    /// The sized component of [`Self`].
    type Sized: Sized;

    /// Create a reference to [`Self::Uninit`] from an initialized reference.
    ///
    /// See [`Uninit::from_ref`] for more details.
    #[inline(always)]
    #[must_use]
    fn as_uninit(&self) -> &Self::Uninit {
        Self::Uninit::from_ref(self)
    }

    /// Create a mutable reference to [`Self::Uninit`] from an initialized mutable reference.
    ///
    /// See [`Uninit::from_mut`] for more details and safety concerns.
    #[inline(always)]
    #[must_use]
    unsafe fn as_uninit_mut(&mut self) -> &mut Self::Uninit {
        unsafe { Self::Uninit::from_mut(self) }
    }
}

impl<T> Init for T {
    type Uninit = MaybeUninit<T>;
    type Sized = T;
}

impl<T> Init for [T] {
    type Uninit = [MaybeUninit<T>];
    type Sized = T;
}
