use std::{
	fmt,
	io::{self, Write as _},
};

use anyhow::Error as AnyError;

#[derive(Debug)]
pub struct CliError {
	message: String,
}

impl CliError {
	pub fn new(message: impl Into<String>) -> Self {
		Self { message: message.into() }
	}

	pub fn unknown_option(option: &str) -> Self {
		Self::new(format!("unexpected argument '{option}' found"))
	}
}

impl fmt::Display for CliError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.message)
	}
}

impl std::error::Error for CliError {}

impl From<lexopt::Error> for CliError {
	fn from(error: lexopt::Error) -> Self {
		Self::new(error.to_string())
	}
}

pub fn report_usage(error: &CliError) {
	let mut stderr = io::stderr().lock();
	let _ = writeln!(stderr, "error: {error}\n\nFor more information, try '--help'.");
}

pub fn is_broken_pipe(error: &AnyError) -> bool {
	error.chain().any(|cause| {
		cause.downcast_ref::<io::Error>().is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
			// serde_json wraps the I/O error without exposing it as a source.
			|| cause.downcast_ref::<serde_json::Error>().and_then(serde_json::Error::io_error_kind)
				== Some(io::ErrorKind::BrokenPipe)
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn broken_pipe_is_recognized_through_wrapping() {
		use anyhow::Context as _;
		let pipe = || io::Error::from(io::ErrorKind::BrokenPipe);
		assert!(is_broken_pipe(&pipe().into()));
		assert!(is_broken_pipe(&Err::<(), _>(pipe()).context("writing report").unwrap_err()));
		assert!(is_broken_pipe(&serde_json::Error::io(pipe()).into()));
		assert!(!is_broken_pipe(&io::Error::from(io::ErrorKind::NotFound).into()));
		assert!(!is_broken_pipe(&anyhow::anyhow!("Skipped 3 entries due to errors")));
	}
}
