#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic, clippy::perf)]
#![deny(warnings)]

#[cfg(target_os = "windows")]
compile_error!("loc does not support Windows; use the Linux (musl static) or macOS builds");

pub mod analysis;
pub mod cli;
pub mod config;
pub mod config_discovery;
pub mod display;
pub mod init;
pub mod langs;
pub mod swar;

/// Run the command line arguments the process was started with.
#[must_use]
pub fn run() -> std::process::ExitCode {
	cli::run(std::env::args_os())
}
