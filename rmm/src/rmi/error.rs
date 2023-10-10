use crate::{measurement::MeasurementError, rsi};

// B3.4.1 RmiCommandReturnCode type
// Default index is 0
#[derive(Debug)]
pub enum Error {
    RmiErrorInput,
    RmiErrorRealm(usize),
    RmiErrorRec,
    RmiErrorRtt(usize),
    RmiErrorInUse,
    RmiErrorCount,
    //// The below are our-defined errors not in TF-RMM
    RmiErrorOthers(InternalError),
}

#[derive(Debug)]
pub enum InternalError {
    NotExistRealm,
    NotExistVCPU,
    MeasurementError,
    InvalidMeasurementIndex,
}

impl From<Error> for usize {
    fn from(err: Error) -> Self {
        match err {
            Error::RmiErrorInput => 1,
            Error::RmiErrorRealm(index) => 2 | (index << 8),
            Error::RmiErrorRec => 3,
            Error::RmiErrorRtt(level) => 4 | (level << 8),
            Error::RmiErrorInUse => 5,
            Error::RmiErrorCount => 6,
            Error::RmiErrorOthers(_) => 7,
        }
    }
}

impl From<vmsa::error::Error> for Error {
    fn from(_e: vmsa::error::Error) -> Self {
        //error!("MmError occured: {}", <Error as Into<usize>>::into(e));
        Error::RmiErrorInput
    }
}

impl From<MeasurementError> for Error {
    fn from(_value: MeasurementError) -> Self {
        Error::RmiErrorOthers(InternalError::MeasurementError)
    }
}

impl From<rsi::error::Error> for Error {
    fn from(value: rsi::error::Error) -> Self {
        match value {
            rsi::error::Error::RealmDoesNotExists => {
                Self::RmiErrorOthers(InternalError::NotExistRealm)
            }
            _ => Self::RmiErrorOthers(InternalError::InvalidMeasurementIndex),
        }
    }
}
