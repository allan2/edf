use std::path::PathBuf;

use clap::Parser;
use reader::Reader;

mod error;
mod reader;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// The input file
	#[clap(short, long, parse(from_os_str), value_name = "INPUT_FILE")]
	input: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();

	let path = args.input;
	Reader::from_path(path)?;
	Ok(())
}
