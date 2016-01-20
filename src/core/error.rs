use std::result;
use std::io;

#[derive(Debug)]
pub enum Error {
    WriteError(String),
    IOError(io::ErrorKind, String),
    FileNotFound(String),
    ReadOnly(String),
    CannotAcquireLock(String),
    FSTFormat(String),
}

pub type Result<T> = result::Result<T, Error>;
