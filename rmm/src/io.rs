extern crate alloc;

use alloc::boxed::Box;
use spinning_top::{Spinlock, SpinlockGuard};

pub use crate::error::{Error, ErrorKind};

pub type Result<T> = core::result::Result<T, Error>;

pub trait Device {
    fn initialize(&mut self) -> Result<()>;
    fn initialized(&self) -> bool;
}

pub trait Write {
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

pub trait ConsoleWriter: Device + Write + Send {}

pub struct Stdout {
    device: Option<Box<dyn ConsoleWriter>>,
}

impl Stdout {
    pub const fn new() -> Self {
        Self { device: None }
    }
    pub fn attach(&mut self, mut device: Box<dyn ConsoleWriter>) -> Result<()> {
        if !device.initialized() {
            device.initialize()?;
        }
        self.device.replace(device);
        Ok(())
    }
}

impl Write for Stdout {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.device
            .as_mut()
            .map(|dev| dev.write_all(buf))
            .unwrap_or(Err(Error::new(ErrorKind::NotConnected)))
    }
}

pub fn stdout() -> SpinlockGuard<'static, Stdout> {
    static STDOUT: Spinlock<Stdout> = Spinlock::new(Stdout::new());
    STDOUT.lock()
}

#[cfg(test)]
pub mod test {
    extern crate alloc;
    use crate::io::{ConsoleWriter, Device, Result, Stdout, Write};
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::cell::RefCell;

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
    fn attach_and_ready() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        let mut stdout = Stdout::new();

        assert!(!mock.initialized());

        stdout.attach(mock).ok().unwrap();

        assert!(unsafe { (*mock_ptr).initialized() });
    }

    #[test]
    fn write() {
        let mock = Box::new(MockDevice::new());
        let mock_ptr = mock.as_ref() as *const MockDevice;
        let mut stdout = Stdout::new();

        stdout.attach(mock).ok().unwrap();

        stdout.write_all("Hello ".as_bytes()).ok().unwrap();
        stdout.write_all("World!".as_bytes()).ok().unwrap();
        assert_eq!(unsafe { (*mock_ptr).output() }, "Hello World!");
    }
}
