use crate::aarch64::ESR_EL2;

#[derive(Debug, Copy, Clone)]
pub enum Fault {
    AddressSize { level: u8 },
    Translation { level: u8 },
    AccessFlag { level: u8 },
    Permission { level: u8 },
    Alignment,
    TLBConflict,
    Other(u8),
}

impl From<u32> for Fault {
    fn from(origin: u32) -> Self {
        let level = (origin & 0b11) as u8;

        match (origin & ESR_EL2::DFSC as u32) >> 2 {
            0b0000 => Fault::AddressSize { level },
            0b0001 => Fault::Translation { level },
            0b0010 => Fault::AccessFlag { level },
            0b0011 => Fault::Permission { level },
            0b1000 => Fault::Alignment,
            0b1100 => Fault::TLBConflict,
            _ => Fault::Other((origin & ESR_EL2::DFSC as u32) as u8),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    PCAlignmentFault,
    DataAbort(Fault),
    SPAlignmentFault,
    Brk(u16),
    HVC,
    Other(u32),
}

impl From<u32> for Syndrome {
    fn from(origin: u32) -> Self {
        match (origin & ESR_EL2::EC as u32) >> ESR_EL2::EC.trailing_zeros() {
            0b00_0000 => Syndrome::Unknown,
            0b01_0010 => Syndrome::HVC,
            0b01_0110 => Syndrome::HVC,
            0b10_0010 => Syndrome::PCAlignmentFault,
            0b10_0100 | 0b10_0101 => Syndrome::DataAbort(Fault::from(origin)),
            0b10_0110 => Syndrome::SPAlignmentFault,
            0b11_1100 => Syndrome::Brk((origin & ESR_EL2::ISS_BRK_CMT as u32) as u16),
            ec => Syndrome::Other(ec as u32),
        }
    }
}
