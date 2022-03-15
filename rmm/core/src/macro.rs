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
    use alloc::boxed::Box;

    extern crate alloc;

    #[test]
    fn println_without_arg() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        stdout().attach(mock).ok().unwrap();

        println!();

        assert_eq!(unsafe { (*mock_ptr).output() }, "\n");
    }

    #[test]
    fn println_without_format() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        stdout().attach(mock).ok().unwrap();

        println!("hello");
        assert_eq!(unsafe { (*mock_ptr).output() }, "hello\n");
    }

    #[test]
    fn println_with_format() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        stdout().attach(mock).ok().unwrap();

        println!("number {}", 1234);
        assert_eq!(unsafe { (*mock_ptr).output() }, "number 1234\n");
    }

    #[test]
    fn eprintln_without_arg() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        stdout().attach(mock).ok().unwrap();

        eprintln!();
        assert_eq!(unsafe { (*mock_ptr).output() }, "\x1b[0;31m\n\x1b[0m");
    }

    #[test]
    fn eprintln_without_format() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        stdout().attach(mock).ok().unwrap();

        eprintln!("hello");
        assert_eq!(unsafe { (*mock_ptr).output() }, "\x1b[0;31mhello\n\x1b[0m");
    }

    #[test]
    fn eprintln_with_format() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        stdout().attach(mock).ok().unwrap();

        eprintln!("number {}", 4321);
        assert_eq!(
            unsafe { (*mock_ptr).output() },
            "\x1b[0;31mnumber 4321\n\x1b[0m"
        );
    }
}
