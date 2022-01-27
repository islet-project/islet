#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {};
}

#[macro_export]
macro_rules! println {
    () => {};
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

#[macro_export]
macro_rules! eprint {
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

#[macro_export]
macro_rules! eprintln {
    () => {};
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

#[cfg(test)]
mod test {
	use crate::{println, eprintln};

	#[test]
	fn println_without_arg() {
		println!();
	}

	#[test]
	fn println_without_format() {
		println!("hello");
	}

	#[test]
	fn println_with_format() {
		println!("number {}", 1234);
	}

	#[test]
	fn eprintln_without_arg() {
		eprintln!();
	}

	#[test]
	fn eprintln_without_format() {
		eprintln!("hello");
	}

	#[test]
	fn eprintln_with_format() {
		eprintln!("number {}", 4321);
	}
}
