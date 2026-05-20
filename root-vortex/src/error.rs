pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TypeMismatch { expected: String, given: String },
    BranchNotFound { name: String },
    Decode(crate::rbytes::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Root Error: {:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<crate::rbytes::Error> for Error {
    fn from(e: crate::rbytes::Error) -> Self {
        Error::Decode(e)
    }
}
