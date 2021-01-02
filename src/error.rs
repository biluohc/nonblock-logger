use log::SetLoggerError;
use std::{borrow::Cow, fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Log(SetLoggerError),
    Desc(Cow<'static, str>),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(io) => write!(fmt, "{}", io),
            Error::Log(log) => write!(fmt, "{}", log),
            Error::Desc(desc) => write!(fmt, "{}", desc.as_ref()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(io) => io.source(),
            Error::Log(_) => None,
            Error::Desc(_) => None,
        }
    }
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
