#![allow(incomplete_features)]
#![feature(const_default_impls)]
#![feature(generic_const_exprs)]

#[cfg(debug_assertions)]
#[macro_use]
extern crate std;

#[cfg(debug_assertions)]
macro_rules! debug {
	() => {
		dbg!()
	};
	($val:expr $(,)?) => {
		dbg!($val)
	};
	($($val:expr),+ $(,)?) => {
		($(dbg!($val)),+,)
	};
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
	() => {};
	($val:expr $(,)?) => {{
		let _ = $val;
		$val
	}};
	($($val:expr),+ $(,)?) => {
		($($val),+,)
	};
}

pub mod arena;
