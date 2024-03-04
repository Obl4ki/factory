use factory_lib::error::FactoryError;
use std::{io, str};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Data(#[from] FactoryError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    DotOutputMalformed(#[from] str::Utf8Error),
}
