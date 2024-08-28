macro_rules! assert_unchecked {
    ($cond:expr) => {{ $crate::util::assert_unchecked!($cond, ) }};
    ($cond:expr, $($arg:tt)*) => {{
        if ::core::cfg!(debug_assertions) {
            #[inline(always)]
            const unsafe fn __needs_unsafe() {}
            __needs_unsafe();

            ::core::assert!($cond, $($arg)*)
        } else {
            if !($cond) { ::core::hint::unreachable_unchecked() }
        }
    }};
}

pub(crate) use assert_unchecked;
