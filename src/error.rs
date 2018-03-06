use std::fmt::{Display, Formatter, Result};
use std::str::Utf8Error;
use ruru;

#[derive(Debug)]
pub enum Error {
  Ruru(ruru::result::Error),
  Corvus(String),
  Utf8Error(Utf8Error),
  Nil,
}

impl From<String> for Error {
  fn from(s: String) -> Error {
    Error::Corvus(s)
  }
}

impl<'a> From<&'a str> for Error {
  fn from(s: &'a str) -> Error {
    Error::Corvus(s.into())
  }
}

impl From<ruru::result::Error> for Error {
  fn from(e: ruru::result::Error) -> Error {
    Error::Ruru(e)
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match *self {
      Error::Ruru(ref err) => write!(f, "ruru error: {}", err),
      Error::Corvus(ref err) => write!(f, "Corvus error: {}", err),
      Error::Nil => write!(f, "nil value passed to nu"),
      Error::Utf8Error(err) => write!(f, "nu string: {}", err),
    }
  }
}
