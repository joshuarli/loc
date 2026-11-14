#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic, clippy::perf)]
#![deny(warnings)]

use std::process::ExitCode;

fn main() -> ExitCode {
	codestats::run()
}
