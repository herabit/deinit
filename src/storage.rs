use core::slice;
use std::mem::MaybeUninit;

use crate::{error::TryReserveError, util::assert_unchecked};

/// Trait to some contiguous buffer of `Item`s.
pub unsafe trait Storage<Item> {
    /// Return a pointer to the base of this buffer.
    ///
    /// # Safety
    ///
    /// - The returned pointer must never be null.
    /// - The returned pointer must always be properly aligned for [`Item`].
    #[must_use]
    fn base_ptr(&self) -> *const Item;

    /// Return a mutable pointer to the base of this buffer.
    ///
    /// # Safety
    ///
    /// - The returned pointer must never be null.
    /// - The returned pointer must always be properly aligned for [`Item`].
    #[must_use]
    fn base_ptr_mut(&mut self) -> *mut Item;

    /// Return the current capacity of the buffer.
    #[must_use]
    fn capacity(&self) -> usize;

    /// Attempt to grow the buffer.
    ///
    /// This should only ever be called when we know we need to resize the buffer.
    fn grow(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError>;

    /// Attempt to grow the buffer by an exact amount.
    ///
    /// This should only ever be called when we know we need to resize the buffer.
    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError>;

    /// Deallocate the memory stored within this buffer.
    unsafe fn dealloc(&mut self);

    /// Returns whether this storage buffer needs to grow.
    #[must_use]
    #[inline(always)]
    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.capacity().wrapping_sub(len)
    }

    /// Return a reference to the potentially uninitialized buffer.
    #[must_use]
    #[inline(always)]
    fn as_buffer(&self) -> &[MaybeUninit<Item>] {
        let cap = self.capacity();
        let ptr = self.base_ptr();

        unsafe { slice::from_raw_parts(ptr.cast(), cap) }
    }

    /// Return a mutable reference to the potentially uninitialized buffer.
    #[must_use]
    #[inline(always)]
    fn as_buffer_mut(&mut self) -> &mut [MaybeUninit<Item>] {
        let cap = self.capacity();
        let ptr = self.base_ptr_mut();

        unsafe { slice::from_raw_parts_mut(ptr.cast(), cap) }
    }

    /// Attempt to reserve additional memory.
    #[must_use]
    #[inline(always)]
    fn try_reserve(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow(len, additional)?;
        }

        unsafe { assert_unchecked!(!self.needs_to_grow(len, additional)) };

        Ok(())
    }

    /// Attempt to reserve an exact amount of additional memory.
    #[must_use]
    #[inline(always)]
    fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional)?;
        }

        unsafe { assert_unchecked!(!self.needs_to_grow(len, additional)) };

        Ok(())
    }
}

unsafe impl<T, S: Storage<T> + ?Sized> Storage<T> for &mut S {
    #[inline(always)]
    fn base_ptr(&self) -> *const T {
        S::base_ptr(self)
    }

    #[inline(always)]
    fn base_ptr_mut(&mut self) -> *mut T {
        S::base_ptr_mut(self)
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        S::capacity(self)
    }

    #[inline(always)]
    fn grow(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        S::grow(self, len, additional)
    }

    #[inline(always)]
    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        S::grow_exact(self, len, additional)
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self) {
        unsafe { S::dealloc(self) }
    }
}

unsafe impl<T, const N: usize> Storage<T> for [MaybeUninit<T>; N] {
    #[inline(always)]
    fn base_ptr(&self) -> *const T {
        self.as_ptr().cast()
    }

    #[inline(always)]
    fn base_ptr_mut(&mut self) -> *mut T {
        self.as_mut_ptr().cast()
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }

    #[inline(always)]
    fn grow(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        self.as_mut_slice().grow(len, additional)
    }

    #[inline(always)]
    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        self.as_mut_slice().grow_exact(len, additional)
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self) {}
}

unsafe impl<T> Storage<T> for [MaybeUninit<T>] {
    #[inline(always)]
    fn base_ptr(&self) -> *const T {
        self.as_ptr().cast()
    }

    #[inline(always)]
    fn base_ptr_mut(&mut self) -> *mut T {
        self.as_mut_ptr().cast()
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn grow(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        self.grow_exact(len, additional)
    }

    #[inline(always)]
    fn grow_exact(&mut self, _: usize, _: usize) -> Result<(), TryReserveError> {
        // If we ever get to this point then we're going to overflow the length of the slice.
        Err(TryReserveError::CapacityOverflow)
    }

    #[inline(always)]
    fn as_buffer(&self) -> &[MaybeUninit<T>] {
        self
    }

    #[inline(always)]
    fn as_buffer_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self) {}
}
