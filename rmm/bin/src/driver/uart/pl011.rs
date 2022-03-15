extern crate alloc;

use alloc::boxed::Box;
use spinning_top::Spinlock;

use rmm_core::io::{self, ConsoleWriter, Error, ErrorKind, Result, Write};

const V2M_OFFSET: usize = 0;
const V2M_IOFPGA_UART3_BASE: usize = V2M_OFFSET + 0x1c0c0000usize;

const BASE: usize = V2M_IOFPGA_UART3_BASE;
const CLK_IN_HZ: usize = 24000000;
const BAUDRATE: usize = 115200;

const REG_LEN: isize = core::mem::size_of::<u32>() as isize;

const UARTDR: isize = 0x000 / REG_LEN;
#[allow(dead_code)]
const UARTRSR: isize = 0x004 / REG_LEN;
const UARTECR: isize = 0x004 / REG_LEN;
#[allow(dead_code)]
const UARTFR: isize = 0x018 / REG_LEN;
#[allow(dead_code)]
const UARTIMSC: isize = 0x038 / REG_LEN;
#[allow(dead_code)]
const UARTRIS: isize = 0x03C / REG_LEN;
#[allow(dead_code)]
const UARTICR: isize = 0x044 / REG_LEN;

#[allow(dead_code)]
const UARTILPR: isize = 0x020 / REG_LEN;
const UARTIBRD: isize = 0x024 / REG_LEN;
const UARTFBRD: isize = 0x028 / REG_LEN;
const UARTLCR_H: isize = 0x02C / REG_LEN;
const UARTCR: isize = 0x030 / REG_LEN;
#[allow(dead_code)]
const UARTIFLS: isize = 0x034 / REG_LEN;
#[allow(dead_code)]
const UARTMIS: isize = 0x040 / REG_LEN;
#[allow(dead_code)]
const UARTDMACR: isize = 0x048 / REG_LEN;

const UARTFR_TXFF_BIT: u32 = 5;

#[allow(dead_code)]
enum UARTCR {
    CTSEN = 1 << 15, /* CTS hardware flow control enable */
    RTSEN = 1 << 14, /* RTS hardware flow control enable */
    RTS = 1 << 11,   /* Request to send */
    DTR = 1 << 10,   /* Data transmit ready. */
    RXE = 1 << 9,    /* Receive enable */
    TXE = 1 << 8,    /* Transmit enable */
    LBE = 1 << 7,    /* Loopback enable */
    EN = 1 << 0,     /* UART Enable */
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum UARTLCR_H {
    SPS = 1 << 7, /* Stick parity select */
    WLEN_8 = 3 << 5,
    WLEN_7 = 2 << 5,
    WLEN_6 = 1 << 5,
    WLEN_5 = 0 << 5,
    FEN = 1 << 4,  /* FIFOs Enable */
    STP2 = 1 << 3, /* Two stop bits select */
    EPS = 1 << 2,  /* Even parity select */
    PEN = 1 << 1,  /* Parity Enable */
    BRK = 1 << 0,  /* Send break */
}

const LINE_CONTROL: u32 = UARTLCR_H::FEN as u32 | UARTLCR_H::WLEN_8 as u32;

struct DeviceInner {
    register: *mut u32,
    ready: bool,
}

impl DeviceInner {
    pub const fn new() -> Self {
        Self {
            register: BASE as *mut u32,
            ready: false,
        }
    }

    pub fn putc(&mut self, byte: u8) -> Result<()> {
        if self.ready {
            unsafe {
                while self.register.offset(UARTFR).read_volatile() & UARTFR_TXFF_BIT == 0 {}
                self.register.offset(UARTDR).write_volatile(byte as u32);
            }
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotConnected))
        }
    }
}

impl io::Device for DeviceInner {
    fn initialized(&self) -> bool {
        self.ready
    }

    fn initialize(&mut self) -> Result<()> {
        if !self.ready {
            unsafe {
                //Disable uart before programming
                self.register.offset(UARTCR).write_volatile(
                    self.register.offset(UARTCR).read_volatile() & !(UARTCR::EN as u32),
                );

                //Program the baudrate
                let divisor = (CLK_IN_HZ << 2) / BAUDRATE;
                let ibrd = (divisor >> 6) as u32;
                self.register.offset(UARTIBRD).write_volatile(ibrd);

                //Write the FBRD
                let fbrd = (ibrd & 0x3f) as u32;
                self.register.offset(UARTFBRD).write_volatile(fbrd);

                self.register.offset(UARTLCR_H).write_volatile(LINE_CONTROL);

                //Clear any pending errors
                self.register.offset(UARTECR).write_volatile(0);

                //Enable tx, rx, and uart overall */
                self.register
                    .offset(UARTCR)
                    .write_volatile(UARTCR::RXE as u32 | UARTCR::TXE as u32 | UARTCR::EN as u32);
            }

            self.ready = true;

            Ok(())
        } else {
            Err(Error::new(ErrorKind::AlreadyExists))
        }
    }
}

impl Write for DeviceInner {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        for byte in buf {
            //Prepand '\r' to '\n'
            if *byte == 0xa {
                self.putc(0xd)?;
            }
            self.putc(*byte)?;
        }
        Ok(())
    }
}

unsafe impl Send for DeviceInner {}

static DEVICE_INNER: Spinlock<DeviceInner> = Spinlock::new(DeviceInner::new());

pub struct Device {}

impl Device {
    pub const fn new() -> Self {
        Self {}
    }
}

impl io::Device for Device {
    fn initialized(&self) -> bool {
        DEVICE_INNER.lock().initialized()
    }

    fn initialize(&mut self) -> Result<()> {
        DEVICE_INNER.lock().initialize()
    }
}

impl Write for Device {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        DEVICE_INNER.lock().write_all(buf)
    }
}

impl ConsoleWriter for Device {}

pub fn device() -> Box<Device> {
    Box::new(Device::new())
}
