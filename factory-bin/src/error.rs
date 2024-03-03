use factory_lib::error::DataError;
use std::{io, str};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Data(#[from] DataError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    DotOutputMalformed(#[from] str::Utf8Error),
    #[error("Failed to open stdio pipe")]
    StdioPipe,
}
