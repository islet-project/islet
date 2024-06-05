// TODO: Expands to cover args, ret
#[macro_export]
macro_rules! define_interface {
    (command {$($variant:ident = $val:expr),*,}) => {
        $(pub const $variant: usize = $val;)*
        pub fn to_str(code: usize) ->  alloc::string::String {
            use alloc::string::ToString;
            use alloc::format;
            match code {
                $($variant => stringify!($variant).to_string()),*,
                _ =>  format!("Undefined {}", code)
            }
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let buffer = alloc::format!($($arg)*);
        let _ = io::stdout().write_all(buffer.as_bytes());
    };
}

#[macro_export]
macro_rules! println {
    () => {crate::print!("\n")};
    ($fmt:expr) => {crate::print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {crate::print!(concat!($fmt, "\n"), $($arg)*)};
}

#[macro_export]
macro_rules! eprint {
    ($fmt:expr) => {
        let buffer = concat!("\x1b[0;31m", $fmt, "\x1b[0m");
        let _ = io::stdout().write_all(buffer.as_bytes());
    };
    ($fmt:expr, $($arg:tt)*) => {{
        let buffer = alloc::format!(concat!("\x1b[0;31m", $fmt, "\x1b[0m"), $($arg)*);
        let _ = io::stdout().write_all(buffer.as_bytes());
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => {crate::eprint!("\n")};
    ($fmt:expr) => {crate::eprint!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {crate::eprint!(concat!($fmt, "\n"), $($arg)*)};
}

#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => {
        // Causes overflow if condition is false
        let _ = [(); 0 - (!($cond) as usize)];
    };
}

#[macro_export]
macro_rules! const_assert_eq {
    ($left:expr, $right:expr) => {
        const _: () = {
            crate::const_assert!($left == $right);
        };
    };
}

#[macro_export]
macro_rules! const_assert_size {
    ($struct:ty, $size:expr) => {
        crate::const_assert_eq!(core::mem::size_of::<$struct>(), ($size));
    };
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use crate::{eprintln, println};
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::cell::RefCell;
    use io::{stdout, Write as IoWrite};
    use io::{ConsoleWriter, Device, Result, Write};

    pub struct MockDevice {
        buffer: RefCell<Vec<u8>>,
        ready: bool,
    }

    impl MockDevice {
        pub const fn new() -> Self {
            MockDevice {
                buffer: RefCell::new(Vec::new()),
                ready: false,
            }
        }

        pub fn output(&self) -> String {
            String::from_utf8(self.buffer.borrow().to_vec()).unwrap()
        }

        pub fn clear(&mut self) {
            self.buffer.borrow_mut().clear()
        }
    }

    impl Device for MockDevice {
        fn initialize(&mut self) -> Result<()> {
            self.ready = true;
            Ok(())
        }

        fn initialized(&self) -> bool {
            self.ready
        }
    }

    impl Write for MockDevice {
        fn write_all(&mut self, buf: &[u8]) -> Result<()> {
            self.buffer.borrow_mut().extend_from_slice(buf);
            Ok(())
        }
    }

    impl ConsoleWriter for MockDevice {}

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

    #[test]
    fn set_of_const_assert() {
        const_assert!(1 != 2);
        const_assert!(true);

        const_assert_eq!(1, 1);
        const_assert_eq!(false, false);

        const_assert_size!(u32, 4);
        const_assert_size!(u64, 8);
    }
}
