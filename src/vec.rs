use core::{
    mem::MaybeUninit,
    ptr::{self},
    slice,
};

use crate::{assert_unchecked, error::TryReserveError};

/// Trait for implementing vector like data structures.
#[allow(dead_code)]
pub(crate) unsafe trait VecImpl {
    type Item: Sized;

    /// Get the length of the vector.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns whether this vector is empty.
    #[must_use]
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Set the length of the vector.
    #[track_caller]
    unsafe fn set_len(&mut self, len: usize);

    /// Get the capacity of the vector.
    #[must_use]
    fn capacity(&self) -> usize;

    /// Attempt to grow the internal buffer.
    fn grow(&mut self, additional: usize) -> Result<(), TryReserveError>;

    /// Attempt to grow the internal buffer exactly.
    fn grow_exact(&mut self, additional: usize) -> Result<(), TryReserveError>;

    /// Return a raw pointer to the start of the vector's buffer.
    #[must_use]
    fn as_ptr(&self) -> *const Self::Item;

    /// Return a mutable raw pointer to the start of the vector's buffer.
    #[must_use]
    fn as_ptr_mut(&mut self) -> *mut Self::Item;

    /// Get the remaining uninitialized capacity of the vector.
    #[must_use]
    #[inline(always)]
    fn remaining(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Returns whether the internal buffer will need to grow
    /// in order to permit a given amount of additional elements.
    #[inline(always)]
    #[must_use]
    fn needs_to_grow(&self, additional: usize) -> bool {
        additional > self.remaining()
    }

    /// Attempt to reserve additional space in the vector.
    #[inline(always)]
    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(additional) {
            self.grow(additional)?;
        }

        unsafe {
            assert_unchecked(
                !self.needs_to_grow(additional),
                "vector failed to return an error when growing the internal buffer",
            );
        }

        Ok(())
    }

    /// Attempt to reserve additional space in the vector exactly.
    #[inline(always)]
    fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(additional) {
            self.grow_exact(additional)?;
        }

        unsafe {
            assert_unchecked(
                !self.needs_to_grow(additional),
                "vector failed to return an error when growing the internal buffer",
            );
        }

        Ok(())
    }

    /// Reserve additional space in the vector.
    #[inline(always)]
    #[track_caller]
    fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional).unwrap();
    }

    /// Reserve an exact amount of additional space in the vector.
    #[inline(always)]
    #[track_caller]
    fn reserve_exact(&mut self, additional: usize) {
        self.try_reserve_exact(additional).unwrap();
    }

    /// Return a slice of the vector's entire buffer.
    #[must_use]
    #[inline(always)]
    fn as_buffer(&self) -> &[MaybeUninit<Self::Item>] {
        let len = self.capacity();
        let ptr = self.as_ptr();

        unsafe { slice::from_raw_parts(ptr.cast(), len) }
    }

    /// Return a mutable slice of the vector's entire buffer.
    ///
    /// The caller must ensure no initialized elements are uninitialized.
    #[must_use]
    #[inline(always)]
    fn as_buffer_mut(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        let len = self.capacity();
        let ptr = self.as_ptr_mut();

        unsafe { slice::from_raw_parts_mut(ptr.cast(), len) }
    }

    /// Return a slice of the vector's elements.
    #[must_use]
    #[inline(always)]
    fn as_slice(&self) -> &[Self::Item] {
        let len = self.len();
        let ptr = self.as_ptr();

        unsafe { slice::from_raw_parts(ptr.cast(), len) }
    }

    /// Return a mutable slice of the vector's elements.
    #[must_use]
    #[inline(always)]
    fn as_slice_mut(&mut self) -> &mut [Self::Item] {
        let len = self.len();
        let ptr = self.as_ptr_mut();

        unsafe { slice::from_raw_parts_mut(ptr.cast(), len) }
    }

    /// Return a slice of the remaining uninitialized elements.
    #[must_use]
    #[inline(always)]
    fn as_remaining(&self) -> &[MaybeUninit<Self::Item>] {
        let len = self.len();
        let cap = self.capacity();

        let ptr = unsafe { self.as_ptr().add(len) };
        let len = cap - len;

        unsafe { slice::from_raw_parts(ptr.cast(), len) }
    }

    /// Return a mutable slice of the remaining uninitialized elements.
    #[must_use]
    #[inline(always)]
    fn as_remaining_mut(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        let len = self.len();
        let cap = self.capacity();

        let ptr = unsafe { self.as_ptr_mut().add(len) };
        let len = cap - len;

        unsafe { slice::from_raw_parts_mut(ptr.cast(), len) }
    }

    /// Push an element without checking that it will fit.
    ///
    /// The caller must ensure that this will not overflow the buffer.
    #[inline(always)]
    unsafe fn push_unchecked(&mut self, item: Self::Item) {
        let len = self.len();
        let cap = self.capacity();

        debug_assert!(len < cap, "pushing item will overflow");

        unsafe {
            // Write the item to the buffer.
            self.as_ptr_mut().add(len).write(item);

            // Increment the length.
            self.set_len(len - 1);
        }
    }

    /// Try to push an element.
    #[must_use]
    #[inline(always)]
    fn try_push(&mut self, item: Self::Item) -> Result<(), (Self::Item, TryReserveError)> {
        if self.len() == self.capacity() {
            if let Err(error) = self.grow(1) {
                return Err((item, error));
            }
        }

        // We've grown at least 1 element in size.
        unsafe {
            self.push_unchecked(item);
        }

        Ok(())
    }

    /// Push an element into the vector.
    #[must_use]
    #[inline(always)]
    #[track_caller]
    fn push(&mut self, item: Self::Item) {
        self.try_push(item).map_err(|(_, error)| error).unwrap();
    }

    /// Pop an element from the vector without checking that it exists.
    #[must_use]
    #[inline(always)]
    unsafe fn pop_unchecked(&mut self) -> Self::Item {
        let len = self.len();

        debug_assert!(len > 0, "popping item will underflow");

        unsafe {
            let len = len - 1;
            self.set_len(len);

            let item = self.as_ptr_mut().add(len).read();

            item
        }
    }

    /// Pop an element from the vector.
    #[must_use]
    #[inline(always)]
    fn pop(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            Some(unsafe { self.pop_unchecked() })
        } else {
            None
        }
    }

    /// Shortens the vector, keeping the first `new_len` elements and drops the rest.
    #[inline(always)]
    fn truncate(&mut self, new_len: usize) {
        let len = self.len();

        if new_len < len {
            unsafe {
                // Update the length before dropping the elements.
                self.set_len(new_len);

                let tail =
                    ptr::slice_from_raw_parts_mut(self.as_ptr_mut().add(new_len), len - new_len);

                tail.drop_in_place();
            }
        }
    }

    /// Clears the vector, dropping all elements.
    #[inline(always)]
    fn clear(&mut self) {
        self.truncate(0);
    }

    #[inline(always)]
    fn retain_mut<F: FnMut(&mut Self::Item) -> bool>(&mut self, mut f: F) {
        let original_len = self.len();
        unsafe { self.set_len(0) };

        // Check the implementation of
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain
        // for safety information.

        struct BackshiftOnDrop<'a, V: VecImpl + ?Sized> {
            v: &'a mut V,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<'a, V: VecImpl + ?Sized> Drop for BackshiftOnDrop<'a, V> {
            #[inline(always)]
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v
                                .as_ptr_mut()
                                .add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }

                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        #[inline(always)]
        fn process_loop<F, V, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<V>,
        ) where
            V: VecImpl + ?Sized,
            F: FnMut(&mut V::Item) -> bool,
        {
            while g.processed_len != original_len {
                let cur = unsafe { &mut *g.v.as_ptr_mut().add(g.processed_len) };

                if !f(cur) {
                    g.processed_len += 1;
                    g.deleted_cnt += 1;

                    unsafe { ptr::drop_in_place(cur) };

                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }

                if DELETED {
                    unsafe {
                        let hole_slot = g.v.as_ptr_mut().add(g.processed_len - g.deleted_cnt);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }

                g.processed_len += 1;
            }
        }

        process_loop::<_, _, false>(original_len, &mut f, &mut g);
        process_loop::<_, _, true>(original_len, &mut f, &mut g);

        drop(g);
    }

    #[inline(always)]
    fn retain<F: FnMut(&Self::Item) -> bool>(&mut self, mut f: F) {
        self.retain_mut(move |item| f(item));
    }
}
