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
