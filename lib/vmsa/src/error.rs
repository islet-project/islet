#[derive(Debug, PartialEq)]
pub enum Error {
    MmStateError,
    MmInvalidAddr,
    MmInvalidLevel,
    MmNoEntry,
    MmAllocFail,
    MmRustError,
    MmUnimplemented,
    MmIsInUse,
    MmRefcountError,
    MmWrongParentChild,
    MmSubtableError,
    MmErrorOthers,
}

impl From<Error> for usize {
    fn from(err: Error) -> Self {
        match err {
            Error::MmStateError => 1,
            Error::MmInvalidAddr => 2,
            Error::MmInvalidLevel => 11,
            Error::MmNoEntry => 12,
            Error::MmAllocFail => 13,
            Error::MmRustError => 14,
            Error::MmUnimplemented => 15,
            Error::MmIsInUse => 16,
            Error::MmRefcountError => 17,
            Error::MmWrongParentChild => 18,
            Error::MmSubtableError => 19,
            Error::MmErrorOthers => 99,
        }
    }
}
