use std::{fmt::Display, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    RuntimeError(rustyscript::Error),
    TypeAnalysisError(serde_reflection::Error),
    DiskIOError {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidElmCall(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn map_disk_error(path: PathBuf) -> impl FnOnce(std::io::Error) -> Error {
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
