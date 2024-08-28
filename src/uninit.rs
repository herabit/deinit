use core::{
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::Deref,
    slice,
};

use crate::{Init, Owned};

mod sealed {
    use core::mem::MaybeUninit;

    pub trait Sealed {}

    impl<T> Sealed for MaybeUninit<T> {}
    impl<T> Sealed for [MaybeUninit<T>] {}
}

/// Trait for types that store potentially uninitialized data.
pub trait Uninit: sealed::Sealed {
    type Init: Init<Uninit = Self> + ?Sized;

    /// Get an uninitialized slice of the underlying data.
    #[must_use]
    fn as_slice(&self) -> &[MaybeUninit<<Self::Init as Init>::Sized>];

    /// Get an uninitialized mutable slice of the underlying data.
    #[must_use]
    fn as_slice_mut(&mut self) -> &mut [MaybeUninit<<Self::Init as Init>::Sized>];

    /// Get a pointer to the potentially uninitialized [`Self::Init`].
    #[must_use]
    fn as_raw(&self) -> *const Self::Init;

    /// Get a mutable pointer to the potentially uninitialized [`Self::Init`].
    #[must_use]
    fn as_raw_mut(&mut self) -> *mut Self::Init;

    /// Get a reference to the [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `self` is in an initialized state.
    #[must_use]
    #[inline(always)]
    unsafe fn assume_init_ref(&self) -> &Self::Init {
        unsafe { &*self.as_raw() }
    }

    /// Get a mutable reference to the [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `self` is in an initialized state.
    #[must_use]
    #[inline(always)]
    unsafe fn assume_init_mut(&mut self) -> &mut Self::Init {
        unsafe { &mut *self.as_raw_mut() }
    }

    /// Get an owned pointer to the [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `self` is in an initialized state.
    #[must_use]
    #[inline(always)]
    unsafe fn assume_init_owned(&mut self) -> Owned<'_, Self::Init> {
        unsafe { Owned::new(self) }
    }

    /// Drop a [`Self::Init`] in place.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `self` is in an initialized state.
    ///
    /// - That the invariants of [`::core::ops::Drop`] for a given [`Self::Init`] or its
    ///   subfields, are upheld.
    #[inline(always)]
    unsafe fn assume_init_drop(&mut self) {
        unsafe { ::core::ptr::drop_in_place(self.as_raw_mut()) }
    }

    /// Reads an initialized [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `self` is in an initialized state.
    ///
    /// - When creating multiple copies of data where [`Self::Init`] does not
    ///   implement [`::core::marker::Copy`], that creating multiple duplicates
    ///   of `self` is indeed valid.
    #[must_use]
    #[inline(always)]
    unsafe fn assume_init_read(&self) -> Self::Init
    where
        Self::Init: Sized,
    {
        unsafe { self.as_raw().read() }
    }

    /// Reads an initialized [`Self::Init`], consuming `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `self` is in an initialized state.
    #[must_use]
    #[inline(always)]
    unsafe fn assume_init(self) -> Self::Init
    where
        Self: Sized,
        Self::Init: Sized,
    {
        let _this = ManuallyDrop::new(self);

        _this.deref().assume_init_read()
    }

    /// Create a new [`Self`] from uninitialized memory.
    #[must_use]
    #[inline(always)]
    fn uninit() -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    /// Create a new [`Self`] from zeroed memory.
    #[must_use]
    #[inline(always)]
    fn zeroed() -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    /// Create a new [`Self`] from initialized memory.
    #[must_use]
    #[inline(always)]
    fn new(_init: Self::Init) -> Self
    where
        Self: Sized,
        Self::Init: Sized,
    {
        unimplemented!()
    }

    /// Create a raw pointer to a [`Self::Init`] from a raw pointer to a [`Self`].
    #[must_use]
    fn raw_from_ptr(ptr: *const Self) -> *const Self::Init;

    /// Create a mutable raw pointer to a [`Self::Init`] from a mutable raw pointer to a [`Self`].
    #[must_use]
    fn raw_from_ptr_mut(ptr: *mut Self) -> *mut Self::Init;

    /// Create a raw pointer to a [`Self`] from a raw pointer to a [`Self::Init`].
    #[must_use]
    fn raw_to_ptr(raw: *const Self::Init) -> *const Self;

    /// Create a mutable raw pointer to a [`Self`] from a mutable raw pointer to a [`Self::Init`].
    #[must_use]
    fn raw_to_ptr_mut(raw: *mut Self::Init) -> *mut Self;

    /// Create a reference to a [`Self`] from a reference to a [`Self::Init`].
    #[must_use]
    #[inline(always)]
    fn from_ref(init: &Self::Init) -> &Self {
        unsafe { &*Self::raw_to_ptr(init) }
    }

    /// Create a mutable reference to a [`Self`] from a mutable reference to a [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the content of `init` is initialized before
    /// the borrow ends.
    #[must_use]
    #[inline(always)]
    unsafe fn from_mut(init: &mut Self::Init) -> &mut Self {
        unsafe { &mut *Self::raw_to_ptr_mut(init) }
    }

    /// Create a reference to a [`Self`] from a raw pointer to a [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - `raw` is a valid pointer to a potentially uninitialized [`Self::Init`].
    ///
    /// - You must enforce Rust's aliasing rules, as `'a` is an arbitrarily chosen
    ///   lifetime.
    #[must_use]
    #[inline(always)]
    unsafe fn from_raw<'a>(raw: *const Self::Init) -> &'a Self {
        unsafe { Self::from_ref(&*raw) }
    }

    /// Create a mutable reference to a [`Self`] from a mutable raw pointer to a [`Self::Init`].
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - `raw` is a valid pointer to a potentially uninitialized [`Self::Init`].
    ///
    /// - You must enforce Rust's aliasing rules, as `'a` is an arbitrarily chosen
    ///   lifetime.
    #[must_use]
    #[inline(always)]
    unsafe fn from_raw_mut<'a>(raw: *mut Self::Init) -> &'a mut Self {
        unsafe { Self::from_mut(&mut *raw) }
    }

    /// Initialize a [`Self`] by filling all of the elements from [`Self::as_slice_mut`] with
    /// a provided value that is cloned.
    ///
    /// # Panics
    ///
    /// This function will panic if cloning the provided value fails.
    ///
    /// Upon panicking, all initialized data will be dropped.
    #[must_use]
    #[inline(always)]
    fn fill_cloned(&mut self, value: <Self::Init as Init>::Sized) -> &mut Self::Init
    where
        <Self::Init as Init>::Sized: Clone,
    {
        let mut guard = Guard {
            slice: self.as_slice_mut(),
            initialized: 0,
        };

        if let Some((last, rest)) = guard.slice.split_last_mut() {
            for elem in rest {
                elem.write(value.clone());
                guard.initialized += 1;
            }

            // Write the final value
            last.write(value);
            // This can probably be removed.
            guard.initialized += 1;
        }

        mem::forget(guard);

        // SAFETY: We've initialized all elements without a panic.
        unsafe { self.assume_init_mut() }
    }

    /// Initialize a [`Self`] by filling all of the elements from [`Self::as_slice_mut`] with
    /// a provided value that is copied.
    ///
    /// This will always succeed.
    #[must_use]
    #[inline(always)]
    fn fill_copied(&mut self, value: <Self::Init as Init>::Sized) -> &mut Self::Init
    where
        <Self::Init as Init>::Sized: Copy,
    {
        <[_]>::fill(self.as_slice_mut(), MaybeUninit::new(value));

        // SAFETY: We're always able to initialize all elements.
        unsafe { self.assume_init_mut() }
    }

    /// Initialize a [`Self`] by filling all of the elements from [`Self::as_slice_mut`] with
    /// the results of a closure.
    ///
    /// # Panics
    ///
    /// This function will panic if the provided closure panics.
    ///
    /// Upon panicking, all initialized data is dropped.
    #[must_use]
    #[inline(always)]
    fn fill_with<F: FnMut() -> <Self::Init as Init>::Sized>(
        &mut self,
        mut f: F,
    ) -> &mut Self::Init {
        // SAFETY: This is always safe as the new closure will always panic
        //         when there is a failure to construct a new element.
        unsafe {
            self.fill_with_in_place(move |elem| {
                elem.write(f());
            })
        }
    }

    /// Initialize a [`Self`] by filling all of the elements from [`Self::as_slice_mut`] with
    /// the results of a closure, in place.
    ///
    /// # Panics
    /// This function will panic if the provided closure panics.
    ///
    /// Upon panicking, all initialized data is dropped.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That the provided closure ***initializes every element***,
    ///   that is unless, it panics.
    ///
    ///   Failure to initialize an element should be considered fatal and
    ///   result in a panic.
    #[must_use]
    #[inline(always)]
    unsafe fn fill_with_in_place<F: FnMut(&mut MaybeUninit<<Self::Init as Init>::Sized>)>(
        &mut self,
        mut f: F,
    ) -> &mut Self::Init {
        let mut guard = Guard {
            slice: self.as_slice_mut(),
            initialized: 0,
        };

        for elem in guard.slice.iter_mut() {
            // SAFETY: The caller ensures that every call to `f` will result in a valid element,
            //         except for panics.
            f(elem);
            guard.initialized += 1;
        }

        mem::forget(guard);

        // SAFETY: We've initialized all elements without a panic.
        unsafe { self.assume_init_mut() }
    }
}

struct Guard<'a, T> {
    slice: &'a mut [MaybeUninit<T>],
    initialized: usize,
}

impl<'a, T> Drop for Guard<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        let init = unsafe { self.slice.get_unchecked_mut(..self.initialized) };

        unsafe { init.assume_init_drop() }
    }
}

impl<T> Uninit for MaybeUninit<T> {
    type Init = T;

    #[inline(always)]
    fn as_raw(&self) -> *const T {
        MaybeUninit::<T>::as_ptr(self)
    }

    #[inline(always)]
    fn as_raw_mut(&mut self) -> *mut T {
        MaybeUninit::<T>::as_mut_ptr(self)
    }

    #[inline(always)]
    fn uninit() -> MaybeUninit<T> {
        MaybeUninit::<T>::uninit()
    }

    #[inline(always)]
    fn zeroed() -> MaybeUninit<T> {
        MaybeUninit::<T>::zeroed()
    }

    #[inline(always)]
    fn new(init: T) -> MaybeUninit<T> {
        MaybeUninit::<T>::new(init)
    }

    #[inline(always)]
    fn from_ref(init: &T) -> &MaybeUninit<T> {
        unsafe { &*(init as *const T as *const MaybeUninit<T>) }
    }

    #[inline(always)]
    unsafe fn from_mut(init: &mut T) -> &mut MaybeUninit<T> {
        unsafe { &mut *(init as *mut T as *mut MaybeUninit<T>) }
    }

    #[inline(always)]
    fn raw_to_ptr(raw: *const T) -> *const MaybeUninit<T> {
        raw as *const MaybeUninit<T>
    }

    #[inline(always)]
    fn raw_to_ptr_mut(raw: *mut T) -> *mut MaybeUninit<T> {
        raw as *mut MaybeUninit<T>
    }

    #[inline(always)]
    fn raw_from_ptr(ptr: *const MaybeUninit<T>) -> *const T {
        ptr as *const T
    }

    #[inline(always)]
    fn raw_from_ptr_mut(ptr: *mut MaybeUninit<T>) -> *mut T {
        ptr as *mut T
    }

    #[inline(always)]
    fn as_slice(&self) -> &[MaybeUninit<T>] {
        slice::from_ref(self)
    }

    #[inline(always)]
    fn as_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        slice::from_mut(self)
    }
}

impl<T> Uninit for [MaybeUninit<T>] {
    type Init = [T];

    #[inline(always)]
    fn as_raw(&self) -> *const [T] {
        self as *const [MaybeUninit<T>] as *const [T]
    }

    #[inline(always)]
    fn as_raw_mut(&mut self) -> *mut [T] {
        self as *mut [MaybeUninit<T>] as *mut [T]
    }

    #[inline(always)]
    fn raw_to_ptr(raw: *const [T]) -> *const [MaybeUninit<T>] {
        raw as *const [MaybeUninit<T>]
    }

    #[inline(always)]
    fn raw_to_ptr_mut(raw: *mut [T]) -> *mut [MaybeUninit<T>] {
        raw as *mut [MaybeUninit<T>]
    }

    #[inline(always)]
    fn raw_from_ptr(ptr: *const [MaybeUninit<T>]) -> *const [T] {
        ptr as *const [T]
    }

    #[inline(always)]
    fn raw_from_ptr_mut(ptr: *mut [MaybeUninit<T>]) -> *mut [T] {
        ptr as *mut [T]
    }

    #[inline(always)]
    fn as_slice(&self) -> &[MaybeUninit<T>] {
        self
    }

    #[inline(always)]
    fn as_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self
    }
}
