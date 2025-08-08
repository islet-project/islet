// TODO: get these from the manifest provided by el3 on rmm's entry
pub const UART_BASE: usize = 0x900_0000;
//pub const UART_BAUDRATE: usize = 115200;
//pub const UART_CLK_IN_HZ: usize = 1;
// Last page of Realm PAS assigned to RMM contains manifest written by EL3
pub const EL3_SHARED_BUF: u64 = 0x428F_F000;
