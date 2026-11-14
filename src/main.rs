#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic, clippy::perf)]
#![deny(warnings)]

fn main() -> anyhow::Result<()> {
	match codestats::run() {
		Err(err) if codestats::is_broken_pipe(&err) => Ok(()),
		result => result,
	}
}
