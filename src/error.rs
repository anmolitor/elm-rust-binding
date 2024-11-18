use std::{fmt::Display, path::PathBuf};

/// Main error type for this crate.
#[derive(Debug)]
pub enum Error {
    /// JavaScript parsing or execution error.
    RuntimeError(rustyscript::Error),
    // Could not infer elm types based on given rust input/output types.
    TypeAnalysisError(serde_reflection::Error),
    // Failed to read/write/delete files.
    DiskIOError {
        path: PathBuf,
        source: std::io::Error,
    },
    // The qualified function name had the wrong format, or the Elm code did not compile.
    InvalidElmCall(String),
}

/// A simple Result alias with the crate specific `Error` type.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn map_disk_error(path: PathBuf) -> impl FnOnce(std::io::Error) -> Error {
        |source| Error::DiskIOError { path, source }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RuntimeError(error) => error.fmt(f),
            Error::TypeAnalysisError(error) => error.fmt(f),
            Error::DiskIOError { path, source } => {
                f.write_fmt(format_args!("DiskIOError at {path:?}: {source}"))
            }
            Error::InvalidElmCall(function_name) => f.write_fmt(format_args!("Invalid Elm Call {function_name}. Expected format is MyModule.MySubmodule.myMethod."))
        }
    }
}

impl std::error::Error for Error {}

impl From<rustyscript::Error> for Error {
    fn from(value: rustyscript::Error) -> Self {
        Error::RuntimeError(value)
    }
}

impl From<serde_reflection::Error> for Error {
    fn from(value: serde_reflection::Error) -> Self {
        Error::TypeAnalysisError(value)
    }
}
