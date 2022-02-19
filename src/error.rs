use std::{error::Error as StdError, fmt, io, result, str};

/// A type alias for `Result<T, edf::Error>`
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur when processing EDF data.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
	/// A crate private constructor for `Error`.
	pub(crate) fn new(kind: ErrorKind) -> Error {
		Error(Box::new(kind))
	}
}

/// The specific type of an error.
#[derive(Debug)]
pub enum ErrorKind {
	/// An I/O error that occurred while reading EDF data.
	Io(io::Error),
	Utf8(str::Utf8Error),
	Header(HeaderError),
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::new(ErrorKind::Io(err))
	}
}

impl From<str::Utf8Error> for Error {
	fn from(err: str::Utf8Error) -> Error {
		Error::new(ErrorKind::Utf8(err))
	}
}

impl StdError for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self.0 {
			ErrorKind::Io(ref err) => err.fmt(f),
			ErrorKind::Utf8(ref err) => err.fmt(f),
			ErrorKind::Header(ref err) => err.fmt(f),
		}
	}
}

/// An error that occured while reading the header.
#[derive(Debug)]
pub enum HeaderError {
	InvalidVersion,
}

impl StdError for HeaderError {}

impl fmt::Display for HeaderError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "invalid version")
	}
}
