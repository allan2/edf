use crate::error::{Error, ErrorKind, HeaderError, Result};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::result;

pub struct Reader;

impl Reader {
	pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Header> {
		let f = File::open(path)?;
		let hdr = Reader::read_header(&f)?;
		Ok(hdr)
	}

	/// Reads and validates the header.
	fn read_header(f: &File) -> Result<Header> {
		Reader::read_version(&f)?;
		let patient_info = Reader::read_patient_info(f)?;
		let recording_id = Reader::read_recording_id(f)?;
		let start_date = Reader::read_start_date(f)?;
		let start_time = Reader::read_start_time(f)?;
		let size = Reader::read_header_size(f)?;
		let reserved = Reader::read_reserved(f)?;
		let records_len = Reader::read_records_len(f)?;
		let duration = Reader::read_duration(f)?;
		let signals_len = Reader::read_signals_len(f)?;
		Ok(Header::new(
			patient_info,
			recording_id,
			start_date,
			start_time,
			size,
			reserved,
			records_len,
			duration,
			signals_len,
		))
	}

	/// Reads and validate the version.
	///
	/// Bytes from 0–80 are the version. The version is always 0.
	fn read_version(mut f: &File) -> Result<()> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		if buffer[0] != 48 {
			return Err(Error::new(ErrorKind::Header(HeaderError::Version)));
		}
		for i in buffer.into_iter().skip(1) {
			if i != 32 {
				return Err(Error::new(ErrorKind::Header(HeaderError::Version)));
			}
		}
		Ok(())
	}

	/// Reads patient information.
	fn read_patient_info(mut f: &File) -> Result<String> {
		let mut buffer = [0; 80];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		Ok(s)
	}

	/// Reads recording information.
	fn read_recording_id(mut f: &File) -> Result<String> {
		let mut buffer = [0; 80];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		Ok(s)
	}

	/// Reads the start date of the recording.
	fn read_start_date(mut f: &File) -> Result<NaiveDate> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		let date = Reader::parse_start_date(s).expect("Invalid start time");
		Ok(date)
	}

	/// Reads the start time of the recording.
	fn read_start_time(mut f: &File) -> Result<NaiveTime> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		let time = NaiveTime::parse_from_str(&s, "%H.%M.%S").expect("Invalid start time");
		Ok(time)
	}

	/// Reads the number of bytes.
	fn read_header_size(mut f: &File) -> Result<usize> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		let n = s.trim_end().parse().expect("Could not parse header size");
		Ok(n)
	}

	// Parse the start date from a string.
	fn parse_start_date(s: String) -> result::Result<NaiveDate, chrono::ParseError> {
		let date = NaiveDate::parse_from_str(&s, "%d.%m.%y")?;
		// The spec specifies a clipping date of 1985.
		let date = if date.year() < 1985 {
			date.with_year(date.year() + 100)
		} else {
			Some(date)
		}
		.unwrap();
		Ok(date)
	}

	/// Reads the reserved block.
	fn read_reserved(mut f: &File) -> Result<String> {
		let mut buffer = [0; 44];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		Ok(s)
	}

	/// Reads the number of records.
	fn read_records_len(mut f: &File) -> Result<Option<usize>> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		let n = s
			.trim_end()
			.parse::<isize>()
			.expect("Could not parse number of records");
		if n == -1 {
			Ok(None)
		} else if n > 0 {
			Ok(Some(n as usize))
		} else {
			panic!("Record length cannot be negative");
		}
	}

	/// Reads the duration of a data record.
	///
	/// The spec recommends that it is a whole number of seconds.
	fn read_duration(mut f: &File) -> Result<usize> {
		let mut buffer = [0; 8];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		let s = s.trim_end();
		// Check to see if there is a trailing decimal.
		let split = s.split_once(".");
		let n = match split {
			None => s,
			Some((characteristic, mantissa)) => match mantissa.parse::<u8>() {
				Ok(v) => match v {
					// The trailing decimals were just zeroes. Continue.
					0 => characteristic,
					_ => panic!("Unimplemented parsing of float durations"),
				},
				Err(_) => panic!("Could not parse mantissa of duration"),
			},
		}
		.parse()
		.expect("Could not parse duration");
		Ok(n)
	}

	/// Reads the number of signals in the data record.
	fn read_signals_len(mut f: &File) -> Result<u32> {
		let mut buffer = [0; 4];
		f.read_exact(&mut buffer)?;
		let s = String::from_utf8(buffer.to_vec())?;
		let n = s
			.trim_end()
			.parse()
			.expect("Could not parse number of signals");
		Ok(n)
	}
}

pub struct Header {
	pub patient_info: String,
	pub recording_id: String,
	/// The start date and time of the recording/
	pub start_datetime: NaiveDateTime,
	// The number of bytes in the header.
	pub size: usize,
	pub reserved: String,
	// The number of records. If unknown (value is -1), then it is `None`.
	pub records_len: Option<usize>,
	// The duration of a a record in seconds.
	pub duration: usize,
	// The number of signals in the record
	pub signals_len: u32,
}

impl Header {
	pub fn new(
		patient_info: String,
		recording_id: String,
		start_date: NaiveDate,
		start_time: NaiveTime,
		size: usize,
		reserved: String,
		records_len: Option<usize>,
		duration: usize,
		signals_len: u32,
	) -> Self {
		let start_datetime = NaiveDateTime::new(start_date, start_time);
		Self {
			patient_info,
			recording_id,
			start_datetime,
			size,
			reserved,
			records_len,
			duration,
			signals_len,
		}
	}
}

impl fmt::Display for Header {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let records_len = match self.records_len {
			None => "-1".to_string(),
			Some(v) => v.to_string(),
		};

		write!(
			f,
			"\n## Header\n{}\nRecording ID: {}\nStart Time: {}\nSize of header: {} B\nReserved: {}\n{} data records\n{} seconds\n{} signals",
			self.patient_info,
			self.recording_id,
			self.start_datetime,
			self.size,
			self.reserved,
			records_len,
			self.duration,
			self.signals_len
		)
	}
}

#[cfg(test)]
mod tests {
	use chrono::NaiveDate;

	use super::Reader;

	// Check that month and date are in the right order.
	#[test]
	fn parse_start_date_simple() {
		let s = String::from("31.01.01");
		assert_eq!(
			Reader::parse_start_date(s),
			Ok(NaiveDate::from_ymd(2001, 1, 31))
		);
	}

	#[test]
	fn parse_start_date_y2k() {
		let s = String::from("01.01.00");
		assert_eq!(
			Reader::parse_start_date(s),
			Ok(NaiveDate::from_ymd(2000, 1, 1))
		);
	}

	#[test]
	fn parse_start_date_before_clip() {
		let s = String::from("01.01.85");
		assert_eq!(
			Reader::parse_start_date(s),
			Ok(NaiveDate::from_ymd(1985, 1, 1))
		);
	}

	#[test]
	fn parse_start_date_after_clip() {
		let s = String::from("31.12.84");
		assert_eq!(
			Reader::parse_start_date(s),
			Ok(NaiveDate::from_ymd(2084, 12, 31))
		);
	}
}
