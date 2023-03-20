extern "C" {
	/// This function doesn't actually exist. It ensures a linking error if it isn't optimized-out.
	pub fn rust_panic_called_where_shouldnt() -> !;
}

/// This macro doesn't panic. Instead it tries to call a non-existing function. If the compiler can
/// prove it can't be called and optimizes it away, the code will compile just fine. Otherwise you get
/// a linking error.
///
/// This should be used only in cases you are absolutely sure are OK and optimizable by compiler.
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dont_panic {
	($($x:tt)*) => {{
		unsafe {
			return crate::macros::rust_panic_called_where_shouldnt();
		}
	}};
}

/// This macro is active only with `panic` feature turned on and it will really panic, instead of
/// causing a linking error. The purpose is to make development easier. (E.g. in debug mode.)
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dont_panic {
    ($($x:tt)*) => ({
        panic!($($x)*);
    })
}
