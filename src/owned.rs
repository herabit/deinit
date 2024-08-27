use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    future::Future,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
};

use crate::{Init, Uninit};

/// An owned pointer to a `T` that does not own the underlying memory.
///
/// # Drops
///
/// Upon a [`Owned`] going out of scope, the `T` within the underlying memory
/// is dropped.
#[repr(transparent)]
pub struct Owned<'a, T: 'a + ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'a + ?Sized> Owned<'a, T> {
    /// Create a new [`Owned`] from a raw pointer to a `T`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - `raw` must be initialized with a valid `T`.
    /// - `raw` must be properly aligned.
    /// - `raw` must not be null.
    /// - `raw` is not aliased for `'a`.
    ///
    /// This can be used as an escape hatch for when a given type does not
    /// implement [`Init`]. Trait objects are one such example.
    ///
    /// However, it is usually preferred to use [`Owned::new`] for types
    /// that implement [`Init`].
    ///
    /// This is because we explicitly take in a reference that upon the
    /// resulting [`Owned`] going out of scope, will always be valid for uninitialized data.
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_raw(raw: *mut T) -> Owned<'a, T> {
        // SAFETY: The caller ensures `raw` is a valid pointer to a `&'a mut T`.
        unsafe {
            Owned {
                ptr: NonNull::new_unchecked(raw),
                _marker: PhantomData,
            }
        }
    }

    /// Leak the value stored in an [`Owned`] and return a raw pointer to a `T`.
    ///
    /// You can reconstruct an [`Owned`] using [`Owned::from_raw`].
    #[inline(always)]
    #[must_use]
    pub fn into_raw(this: Owned<'a, T>) -> *mut T {
        let this = ManuallyDrop::new(this);
        this.ptr.as_ptr()
    }

    /// Get a raw pointer to a `T` from an [`Owned`].
    #[inline(always)]
    #[must_use]
    pub const fn as_raw(this: &Owned<'a, T>) -> *const T {
        this.ptr.as_ptr()
    }

    /// Get a mutable raw pointer to a `T` from an [`Owned`].
    #[inline(always)]
    #[must_use]
    pub fn as_raw_mut(this: &mut Owned<'a, T>) -> *mut T {
        this.ptr.as_ptr()
    }

    /// Leak the value stored in an [`Owned`] and return a mutable reference to a `T`.
    #[inline(always)]
    #[must_use]
    pub fn leak(this: Owned<'a, T>) -> &'a mut T {
        unsafe { &mut *Owned::into_raw(this) }
    }
}

impl<'a, T: 'a> Owned<'a, T> {
    /// Get the inner `T` from an [`Owned`].
    #[inline(always)]
    #[must_use]
    pub fn into_inner(this: Owned<'a, T>) -> T {
        let this = ManuallyDrop::new(this);
        unsafe { this.ptr.read() }
    }
}

impl<'a, T: 'a + ?Sized + Init> Owned<'a, T> {
    /// Create an [`Owned`] from a mutable reference to an initialized `T`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That the data at `init` is in an initialized state.
    #[inline(always)]
    #[must_use]
    pub unsafe fn new(init: &'a mut T::Uninit) -> Owned<'a, T> {
        // SAFETY: The caller ensures that `init` is initialized.
        let init = unsafe { init.assume_init_mut() };

        Owned {
            ptr: init.into(),
            _marker: PhantomData,
        }
    }

    /// Create an [`Owned`] to an uninitialized `T` from an [`Owned`] to an initialized `T`.
    ///
    /// # Drops
    ///
    /// The returned [`Owned`] will not drop the `T`.
    ///
    /// In other words, the `T` is essentially leaked memory.
    ///
    /// If you're looking to drop the inner `T` and then attain the resulting
    /// uninitialized [`Owned`], use [`Owned::take`].
    #[inline(always)]
    #[must_use]
    pub fn into_uninit(this: Owned<'a, T>) -> Owned<'a, T::Uninit> {
        let ptr = Owned::into_raw(this);
        let ptr = T::Uninit::raw_to_ptr_mut(ptr);

        // SAFETY: `ptr` is a valid pointer to a potentially uninitalized `T`.
        unsafe { Owned::from_raw(ptr) }
    }

    /// Create an [`Owned`] to an uninitialized `T` from an [`Owned`] to an initialized `T`.
    ///
    /// # Drops
    ///
    /// This will drop the stored `T` before returning.
    #[inline(always)]
    #[must_use]
    pub fn take(this: Owned<'a, T>) -> Owned<'a, T::Uninit> {
        let ptr = Owned::into_raw(this);

        // SAFETY: `ptr` is a valid pointer to an initialized `T`.
        unsafe { ptr.drop_in_place() };

        let ptr = T::Uninit::raw_to_ptr_mut(ptr);

        // SAFETY: `ptr` is a valid pointer to an uninitialized `T`.
        unsafe { Owned::from_raw(ptr) }
    }
}

impl<'a, T: 'a + ?Sized + Uninit> Owned<'a, T> {
    /// Create an [`Owned`] to an initialized `T`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `this` is in an initialized state.
    #[inline(always)]
    #[must_use]
    pub unsafe fn assume_init(this: Owned<'a, T>) -> Owned<'a, T::Init> {
        let ptr = Owned::into_raw(this);
        let ptr = T::raw_from_ptr_mut(ptr);

        // SAFETY: Caller ensures `this` is initialized.
        unsafe { Owned::from_raw(ptr) }
    }
}

impl<'a, T: 'a + ?Sized> Drop for Owned<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.ptr.drop_in_place() }
    }
}

impl<'a, T: 'a + ?Sized> Deref for Owned<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T: 'a + ?Sized> DerefMut for Owned<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a, T: 'a + ?Sized> Borrow<T> for Owned<'a, T> {
    #[inline(always)]
    fn borrow(&self) -> &T {
        self
    }
}

impl<'a, T: 'a + ?Sized> BorrowMut<T> for Owned<'a, T> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T: 'a + ?Sized> AsRef<T> for Owned<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<'a, T: 'a + ?Sized> AsMut<T> for Owned<'a, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T: 'a + ?Sized + PartialEq> PartialEq for Owned<'a, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }

    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        self.deref().ne(other.deref())
    }
}

impl<'a, T: 'a + ?Sized + Eq> Eq for Owned<'a, T> {}

impl<'a, T: 'a + ?Sized + PartialOrd> PartialOrd for Owned<'a, T> {
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

impl<'a, T: 'a + ?Sized + Ord> Ord for Owned<'a, T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<'a, T: 'a + ?Sized + Hash> Hash for Owned<'a, T> {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<'a, T: 'a + ?Sized + Hasher> Hasher for Owned<'a, T> {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.deref().finish()
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.deref_mut().write(bytes)
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.deref_mut().write_u8(i)
    }

    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.deref_mut().write_u16(i)
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.deref_mut().write_u32(i)
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.deref_mut().write_u64(i)
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.deref_mut().write_u128(i)
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.deref_mut().write_usize(i)
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) {
        self.deref_mut().write_i8(i)
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) {
        self.deref_mut().write_i16(i)
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) {
        self.deref_mut().write_i32(i)
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) {
        self.deref_mut().write_i64(i)
    }

    #[inline(always)]
    fn write_i128(&mut self, i: i128) {
        self.deref_mut().write_i128(i)
    }

    #[inline(always)]
    fn write_isize(&mut self, i: isize) {
        self.deref_mut().write_isize(i)
    }
}

impl<'a, T: 'a + ?Sized + Iterator> Iterator for Owned<'a, T> {
    type Item = T::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.deref_mut().next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.deref().size_hint()
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.deref_mut().nth(n)
    }

    #[inline(always)]
    fn last(self) -> Option<Self::Item> {
        #[inline(always)]
        fn some<T>(_: Option<T>, x: T) -> Option<T> {
            Some(x)
        }

        self.fold(None, some)
    }
}

impl<'a, T: 'a + ?Sized + FusedIterator> FusedIterator for Owned<'a, T> {}

impl<'a, T: 'a + ?Sized + ExactSizeIterator> ExactSizeIterator for Owned<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.deref().len()
    }
}

impl<'a, T: 'a + ?Sized + DoubleEndedIterator> DoubleEndedIterator for Owned<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deref_mut().next_back()
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.deref_mut().nth_back(n)
    }
}

impl<'a, T: 'a + ?Sized + fmt::Debug> fmt::Debug for Owned<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'a, T: 'a + ?Sized + fmt::Display> fmt::Display for Owned<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'a, T: 'a + ?Sized> fmt::Pointer for Owned<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ptr.fmt(f)
    }
}

impl<'a, T: 'a> Default for Owned<'a, [T]> {
    #[inline(always)]
    fn default() -> Self {
        // SAFETY: It should always be okay to drop an empty slice of anything.
        unsafe { Owned::new([].as_mut_slice()) }
    }
}

impl<'a, T: 'a> Default for Owned<'a, [T; 0]> {
    #[inline(always)]
    fn default() -> Self {
        // SAFETY: It should always be okay to drop an empty array of anything.
        unsafe { Owned::new([].as_uninit_mut()) }
    }
}

impl<'a> Default for Owned<'a, str> {
    #[inline(always)]
    fn default() -> Self {
        // SAFETY: An empty string is just an empty byte slice internally.
        let bytes = Owned::<[u8]>::default();
        let ptr = Owned::into_raw(bytes) as *mut str;
        unsafe { Owned::from_raw(ptr) }
    }
}

impl<'a, F: 'a + ?Sized + Future + Unpin> Future for Owned<'a, F> {
    type Output = F::Output;

    #[inline(always)]
    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        F::poll(Pin::new(&mut *self), cx)
    }
}

impl<'a, T: 'a + ?Sized> Unpin for Owned<'a, T> {}

unsafe impl<'a, T: 'a + ?Sized> Send for Owned<'a, T> where &'a mut T: Send {}
unsafe impl<'a, T: 'a + ?Sized> Sync for Owned<'a, T> where &'a mut T: Sync {}
