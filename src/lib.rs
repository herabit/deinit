#![no_std]
#![allow(unused_unsafe)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod owned;
pub use owned::Owned;

mod uninit;
pub use uninit::Uninit;

mod init;
pub use init::Init;

mod vec_impl;

mod slice_vec;
pub use slice_vec::SliceVec;

pub mod error;

/// Assert that a condition is always true, helping to hint to the optimizer.
#[inline(always)]
const unsafe fn assert_unchecked(cond: bool, msg: &str) {
    if cond {
        return;
    }

    if cfg!(debug_assertions) {
        panic!("{}", msg)
    } else {
        unsafe { ::core::hint::unreachable_unchecked() }
    }
}
