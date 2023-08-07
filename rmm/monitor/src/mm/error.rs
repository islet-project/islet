use crate::rmi::error::Error as RmiError;

#[derive(Debug, PartialEq)]
pub enum Error {
    MmStateError,
    MmInvalidAddr,
    MmInvalidLevel,
    MmNoEntry,
    MmAllocFail,
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
            Error::MmErrorOthers => 99,
        }
    }
}

impl From<Error> for RmiError {
    fn from(_: Error) -> Self {
        RmiError::RmiErrorInput
    }
}
