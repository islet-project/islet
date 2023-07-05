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

pub enum InternalError {
    NotExistRealm,
    NotExistVCPU,
}
