use std::{fmt::Display, path::PathBuf};

/// Main error type for this crate.
#[derive(Debug)]
pub enum Error {
    /// JavaScript parsing or execution error.
    #[cfg(feature = "v8")]
    RuntimeError(rustyscript::Error),
    #[cfg(feature = "quickjs")]
    RuntimeError(quickjs_runtime::jsutils::JsError),
    /// JavaScript parsing or execution error.
    NonF64Number(serde_json::Number),
    SerdeJson(serde_json::Error),
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
pub type Result<T> = std::result::Result<T, Box<Error>>;

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
            Error::InvalidElmCall(function_name) => f.write_fmt(format_args!("Invalid Elm Call {function_name}. Expected format is MyModule.MySubmodule.myMethod.")),
            Error::SerdeJson(error) => error.fmt(f),
            Error::NonF64Number(number) => f.write_fmt(format_args!("Non f64 number {number}")),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "v8")]
impl From<rustyscript::Error> for Box<Error> {
    fn from(value: rustyscript::Error) -> Self {
        Box::new(Error::RuntimeError(value))
    }
}

#[cfg(feature = "quickjs")]
impl From<quickjs_runtime::jsutils::JsError> for Box<Error> {
    fn from(value: quickjs_runtime::jsutils::JsError) -> Self {
        Box::new(Error::RuntimeError(value))
    }
}

#[cfg(feature = "quickjs")]
impl From<quickjs_runtime::values::JsValueFacade> for Box<Error> {
    fn from(value: quickjs_runtime::values::JsValueFacade) -> Self {
        Box::new(Error::RuntimeError(
            quickjs_runtime::jsutils::JsError::new_string(format!(
                "JS threw an Error (most likely a rejected promise): {value:?}"
            )),
        ))
    }
}

impl From<serde_json::Error> for Box<Error> {
    fn from(value: serde_json::Error) -> Self {
        Box::new(Error::SerdeJson(value))
    }
}

impl From<serde_reflection::Error> for Box<Error> {
    fn from(value: serde_reflection::Error) -> Self {
        Box::new(Error::TypeAnalysisError(value))
    }
}
