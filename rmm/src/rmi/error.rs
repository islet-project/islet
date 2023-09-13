#[derive(Debug)]
pub enum Error {
    RmiErrorInput,
    RmiErrorRealm,
    RmiErrorRec,
    RmiErrorRtt,
    RmiErrorInUse,
    RmiErrorCount,
    //// The below are our-defined errors not in TF-RMM
    RmiErrorOthers(InternalError),
}

#[derive(Debug)]
pub enum InternalError {
    NotExistRealm,
    NotExistVCPU,
}

impl From<Error> for usize {
    fn from(err: Error) -> Self {
        match err {
            Error::RmiErrorInput => 1,
            Error::RmiErrorRealm => 2,
            Error::RmiErrorRec => 3,
            Error::RmiErrorRtt => 4,
            Error::RmiErrorInUse => 5,
            Error::RmiErrorCount => 6,
            Error::RmiErrorOthers(_) => 7,
        }
    }
}

impl From<paging::error::Error> for Error {
    fn from(_e: paging::error::Error) -> Self {
        //error!("MmError occured: {}", <Error as Into<usize>>::into(e));
        Error::RmiErrorInput
    }
}
