use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    hash::Hash,
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    slice,
};

use crate::{error::TryReserveError, vec::VecImpl, Init, Owned, Uninit};

pub struct Vec<'a, T> {
    buf: &'a mut [MaybeUninit<T>],
    len: usize,
}

impl<'a, T> Vec<'a, T> {
    /// Create a new [`SliceVec`] from an uninitialized slice.
    #[inline(always)]
    #[must_use]
    pub fn new(slice: &'a mut [MaybeUninit<T>]) -> Vec<'a, T> {
        Vec { buf: slice, len: 0 }
    }
}

unsafe impl<T> VecImpl for Vec<'_, T> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        self.len = len;
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline(always)]
    fn grow(&mut self, _: usize) -> Result<(), TryReserveError> {
        Err(TryReserveError::CapacityOverflow)
    }

    #[inline(always)]
    fn grow_exact(&mut self, _: usize) -> Result<(), TryReserveError> {
        Err(TryReserveError::CapacityOverflow)
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const Self::Item {
        self.buf.as_ptr().cast()
    }

    #[inline(always)]
    fn as_ptr_mut(&mut self) -> *mut Self::Item {
        self.buf.as_mut_ptr().cast()
    }
}

impl<T> Deref for Vec<'_, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for Vec<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T> Borrow<[T]> for Vec<'_, T> {
    #[inline(always)]
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> BorrowMut<[T]> for Vec<'_, T> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> AsRef<[T]> for Vec<'_, T> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for Vec<'_, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Default for Vec<'_, T> {
    #[inline(always)]
    fn default() -> Self {
        Vec::new(&mut [])
    }
}

impl<T: Hash> Hash for Vec<'_, T> {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T: PartialEq> PartialEq for Vec<'_, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }

    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        self.deref().ne(other.deref())
    }
}

impl<T: Eq> Eq for Vec<'_, T> {}

impl<T: PartialOrd> PartialOrd for Vec<'_, T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        self.deref().lt(other.deref())
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        self.deref().le(other.deref())
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        self.deref().gt(other.deref())
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        self.deref().ge(other.deref())
    }
}

impl<T: Ord> Ord for Vec<'_, T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<'b, T> IntoIterator for &'b Vec<'_, T> {
    type Item = &'b T;
    type IntoIter = slice::Iter<'b, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'b, T> IntoIterator for &'b mut Vec<'_, T> {
    type Item = &'b mut T;
    type IntoIter = slice::IterMut<'b, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for Vec<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T> Drop for Vec<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        let elems: *mut [T] = self.as_slice_mut();

        unsafe { elems.drop_in_place() }
    }
}
