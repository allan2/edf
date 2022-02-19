use crate::error::{Error, ErrorKind, HeaderError, Result};
use std::{fs::File, io::Read, path::Path};

pub struct Reader;

impl Reader {
	pub fn from_path<P: AsRef<Path>>(path: P) -> Result<()> {
		let f = File::open(path)?;
		Reader::read_version(f)?;
		Ok(())
	}

	fn read_version(mut f: File) -> Result<()> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		let first = match buffer.first() {
			Some(v) => v,
			None => return Err(Error::new(ErrorKind::Header(HeaderError::InvalidVersion))),
		};
		if first != &48 {
			return Err(Error::new(ErrorKind::Header(HeaderError::InvalidVersion)));
		}
		for i in buffer.into_iter().skip(1) {
			if i != 32 {
				return Err(Error::new(ErrorKind::Header(HeaderError::InvalidVersion)));
			}
		}
		Ok(())
	}
}

pub struct Header {
	version: u8,
	//pub start_date: NaiveDateTime,
}

impl Header {
	pub fn new() -> Self {
		Header { version: 0 }
	}
}
