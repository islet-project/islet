#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    NotConnected,
    AlreadyExists,
    StorageFull,
    Unsupported,
    Other,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}
