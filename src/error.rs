use log::SetLoggerError;
use std::{borrow::Cow, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Log(SetLoggerError),
    Desc(Cow<'static, str>),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<SetLoggerError> for Error {
    fn from(e: SetLoggerError) -> Self {
        Error::Log(e)
    }
}

impl From<&'static str> for Error {
    fn from(str: &'static str) -> Self {
        Error::Desc(str.into())
    }
}

impl From<String> for Error {
    fn from(str: String) -> Self {
        Error::Desc(str.into())
    }
}

impl From<Cow<'static, str>> for Error {
    fn from(str: Cow<'static, str>) -> Self {
        Error::Desc(str)
    }
}
