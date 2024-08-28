use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    hash::Hash,
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    slice,
};

use crate::{error::TryReserveError, vec_impl::VecImpl, Init, Owned, Uninit};

pub struct SliceVec<'a, T> {
    buf: &'a mut [MaybeUninit<T>],
    len: usize,
}

impl<'a, T> SliceVec<'a, T> {
    /// Create a new [`SliceVec`] from an uninitialized slice.
    #[inline(always)]
    #[must_use]
    pub fn new(slice: &'a mut [MaybeUninit<T>]) -> SliceVec<'a, T> {
        SliceVec { buf: slice, len: 0 }
    }

    /// Create a new [`SliceVec`] from an potentially uninitialized slice,
    /// and a length.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - `slice` is initialized for the first `len` elements.
    /// - `len` is less than or equal to the length of the slice.
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_raw_parts(buf: &'a mut [MaybeUninit<T>], len: usize) -> SliceVec<'a, T> {
        unsafe { SliceVec { buf, len } }
    }

    /// Decompose a [`SliceVec`] into its raw components: `(buffer, length)`.
    #[inline(always)]
    #[must_use]
    pub fn into_raw_parts(self) -> (&'a mut [MaybeUninit<T>], usize) {
        let mut this = ManuallyDrop::new(self);

        (mem::take(&mut this.buf), this.len)
    }

    /// Get the length of this vector.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        VecImpl::len(self)
    }

    /// Returns whether this vector is empty.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        VecImpl::is_empty(self)
    }

    /// Get the capacity of this vector.
    #[inline(always)]
    #[must_use]
    pub fn capacity(&self) -> usize {
        VecImpl::capacity(self)
    }

    /// Get the remaining capacity of this vector.
    #[inline(always)]
    #[must_use]
    pub fn remaining(&self) -> usize {
        VecImpl::remaining(self)
    }

    /// Returns whether this vector is full.
    #[inline(always)]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Get a raw pointer to this vector's buffer.
    #[inline(always)]
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        VecImpl::as_ptr(self)
    }

    /// Get a mutable raw pointer to this vector's buffer.
    #[inline(always)]
    #[must_use]
    pub fn as_ptr_mut(&mut self) -> *mut T {
        VecImpl::as_ptr_mut(self)
    }

    /// Get a slice to the initialized elements in this vector.
    #[inline(always)]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        VecImpl::as_slice(self)
    }

    /// Get a mutable slice to the initialized elements in this vector.
    #[inline(always)]
    #[must_use]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        VecImpl::as_slice_mut(self)
    }

    /// Get a slice to the remaining uninitialized elements in this vector.
    #[inline(always)]
    #[must_use]
    pub fn as_remaining(&self) -> &[MaybeUninit<T>] {
        VecImpl::as_remaining(self)
    }

    /// Get a mutable slice to the remaining uninitialized elements in this vector.
    #[inline(always)]
    #[must_use]
    pub fn as_remaining_mut(&mut self) -> &mut [MaybeUninit<T>] {
        VecImpl::as_remaining_mut(self)
    }

    /// Split this vector into its initialized slice, and remaining uninitialized slice.
    #[inline(always)]
    #[must_use]
    pub fn as_parts(&self) -> (&[T], &[MaybeUninit<T>]) {
        let (init, uninit) = unsafe { self.buf.split_at_unchecked(self.len) };
        let init = unsafe { init.assume_init_ref() };

        (init, uninit)
    }

    /// Split this vector mutably into its initialized slice, and remaining uninitialized slice.
    #[inline(always)]
    #[must_use]
    pub fn as_parts_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        let (init, uninit) = unsafe { self.buf.split_at_mut_unchecked(self.len) };
        let init = unsafe { init.assume_init_mut() };

        (init, uninit)
    }

    /// Split this vector into its initialized slice, and remaining uninitialized slice.
    ///
    /// This consumes `self` and the caller takes ownership of the sections of the vector.
    #[inline(always)]
    #[must_use]
    pub fn into_parts(self) -> (Owned<'a, [T]>, Owned<'a, [MaybeUninit<T>]>) {
        let (buf, len) = self.into_raw_parts();

        let (init, uninit) = unsafe { buf.split_at_mut_unchecked(len) };
        let init = unsafe { init.assume_init_owned() };
        let uninit = unsafe { uninit.as_uninit_mut().assume_init_owned() };

        (init, uninit)
    }

    #[inline(always)]
    pub fn truncate(&mut self, new_len: usize) {
        VecImpl::truncate(self, new_len)
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        VecImpl::clear(self)
    }

    #[inline(always)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        VecImpl::set_len(self, new_len)
    }

    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, item: T) {
        unsafe { VecImpl::push_unchecked(self, item) }
    }

    #[inline(always)]
    pub fn try_push(&mut self, item: T) -> Result<(), (T, TryReserveError)> {
        VecImpl::try_push(self, item)
    }

    #[inline(always)]
    #[track_caller]
    pub fn push(&mut self, item: T) {
        VecImpl::push(self, item)
    }

    #[inline(always)]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        VecImpl::try_reserve(self, additional)
    }

    #[inline(always)]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        VecImpl::try_reserve_exact(self, additional)
    }
}

unsafe impl<T> VecImpl for SliceVec<'_, T> {
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

impl<T> Deref for SliceVec<'_, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for SliceVec<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T> Borrow<[T]> for SliceVec<'_, T> {
    #[inline(always)]
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> BorrowMut<[T]> for SliceVec<'_, T> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> AsRef<[T]> for SliceVec<'_, T> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for SliceVec<'_, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Default for SliceVec<'_, T> {
    #[inline(always)]
    fn default() -> Self {
        SliceVec::new(&mut [])
    }
}

impl<T: Hash> Hash for SliceVec<'_, T> {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T: PartialEq> PartialEq for SliceVec<'_, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }

    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        self.deref().ne(other.deref())
    }
}

impl<T: Eq> Eq for SliceVec<'_, T> {}

impl<T: PartialOrd> PartialOrd for SliceVec<'_, T> {
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

impl<T: Ord> Ord for SliceVec<'_, T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<'b, T> IntoIterator for &'b SliceVec<'_, T> {
    type Item = &'b T;
    type IntoIter = slice::Iter<'b, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'b, T> IntoIterator for &'b mut SliceVec<'_, T> {
    type Item = &'b mut T;
    type IntoIter = slice::IterMut<'b, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for SliceVec<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T> Drop for SliceVec<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        let elems: *mut [T] = self.as_slice_mut();

        unsafe { elems.drop_in_place() }
    }
}
