#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    NotConnected,
    AlreadyExists,
    Unsupported,
    Other,
}

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
