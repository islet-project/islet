#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        extern crate alloc;
        use core::fmt::Write;
        use $crate::io::Write as IoWrite;
        let buffer = alloc::format!($($arg)*);
        let _ = unsafe {
			$crate::io::stdout()
		 }.write_all(buffer.as_bytes());
    }};
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($fmt:expr) => {$crate::print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::print!(concat!($fmt, "\n"), $($arg)*)};
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        extern crate alloc;
        use core::fmt::Write;
        use $crate::io::Write as IoWrite;
        let buffer = alloc::format!("\x1b[0;31m{}\x1b[0m", $($arg)*);
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
