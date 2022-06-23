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

impl From<Error> for &'static str {
    fn from(error: Error) -> Self {
        match error.kind() {
            ErrorKind::NotConnected => "Communication error: NotConnected",
            ErrorKind::AlreadyExists => "Communication error: AlreadyExists",
            ErrorKind::StorageFull => "Communication error: StorageFull",
            ErrorKind::Unsupported => "Communication error: Unsupported",
            ErrorKind::Other => "Communication error: Other",
        }
    }
}
