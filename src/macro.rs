#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let buffer = alloc::format!($($arg)*);
        let _ = $crate::io::stdout().write_all(buffer.as_bytes());
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
        let buffer = concat!("\x1b[0;31m", $fmt, "\x1b[0m");
        let _ = $crate::io::stdout().write_all(buffer.as_bytes());

	};
    ($fmt:expr, $($arg:tt)*) => {{
        let buffer = alloc::format!(concat!("\x1b[0;31m", $fmt, "\x1b[0m"), $($arg)*);
        let _ = $crate::io::stdout().write_all(buffer.as_bytes());
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
    use crate::io::test::MockDevice;
    use crate::io::{stdout, Write as IoWrite};
    use crate::{eprintln, println};

    extern crate alloc;

    static mut MOCK: MockDevice = MockDevice::new();

    fn setup() {
        unsafe {
            stdout().attach(&mut MOCK).ok().unwrap();
            MOCK.clear();
        }
    }

    #[test]
    fn println_without_arg() {
        setup();
        println!();
        assert_eq!(unsafe { MOCK.output() }, "\n");
    }

    #[test]
    fn println_without_format() {
        setup();
        println!("hello");
        assert_eq!(unsafe { MOCK.output() }, "hello\n");
    }

    #[test]
    fn println_with_format() {
        setup();
        println!("number {}", 1234);
        assert_eq!(unsafe { MOCK.output() }, "number 1234\n");
    }

    #[test]
    fn eprintln_without_arg() {
        setup();
        eprintln!();
        assert_eq!(unsafe { MOCK.output() }, "\x1b[0;31m\n\x1b[0m");
    }

    #[test]
    fn eprintln_without_format() {
        setup();
        eprintln!("hello");
        assert_eq!(unsafe { MOCK.output() }, "\x1b[0;31mhello\n\x1b[0m");
    }

    #[test]
    fn eprintln_with_format() {
        setup();
        eprintln!("number {}", 4321);
        assert_eq!(unsafe { MOCK.output() }, "\x1b[0;31mnumber 4321\n\x1b[0m");
    }
}
