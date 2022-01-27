#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
		#[allow(unused_extern_crates)]
        extern crate alloc;
		#[allow(unused_imports)]
        use core::fmt::Write;
        use $crate::io::Write as IoWrite;
        let buffer = alloc::format!($($arg)*);
        let _ = unsafe {
			$crate::io::stdout()
		 }.write_all(buffer.as_bytes());
    };
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($fmt:expr) => {$crate::print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::print!(concat!($fmt, "\n"), $($arg)*)};
}

#[macro_export]
macro_rules! eprint {
    ($fmt:expr) => {
		#[allow(unused_extern_crates)]
        extern crate alloc;
		#[allow(unused_imports)]
        use core::fmt::Write;
        use $crate::io::Write as IoWrite;
        let buffer = concat!("\x1b[0;31m", $fmt, "\x1b[0m");
        let _ = unsafe {
			$crate::io::stdout()
		 }.write_all(buffer.as_bytes());

	};
    ($fmt:expr, $($arg:tt)*) => {{
        extern crate alloc;
		#[allow(unused_imports)]
        use core::fmt::Write;
        use $crate::io::Write as IoWrite;
        let buffer = alloc::format!(concat!("\x1b[0;31m", $fmt, "\x1b[0m"), $($arg)*);
        let _ = unsafe {
			$crate::io::stdout()
		 }.write_all(buffer.as_bytes());
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => {$crate::eprint!("\n")};
    ($fmt:expr) => {$crate::eprint!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::eprint!(concat!($fmt, "\n"), $($arg)*)};
}

#[cfg(test)]
mod test {
    use crate::{eprintln, println};

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
