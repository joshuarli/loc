use std::{
	ffi::OsString,
	io::{self, Write as _},
	process::ExitCode,
};

use lexopt::Parser;

use super::{
	command::{self, CommandSpec, Flow},
	error::CliError,
};

pub static LANGS_SPEC: CommandSpec = CommandSpec {
	description: "List all supported programming languages",
	options: &[],
	positionals: &[],
	subcommands: &[],
	conflicts: &[],
	note: None,
};

pub fn run(full_name: &str, argv: Vec<OsString>) -> ExitCode {
	super::exit_with(parse(full_name, argv), |(), _| {
		let mut stdout = io::BufWriter::new(io::stdout().lock());
		let width = terminal_size::terminal_size().map_or(80, |(width, _)| usize::from(width.0));
		crate::langs::print_all_languages(&mut stdout, width)?;
		stdout.flush()?;
		Ok(())
	})
}

fn parse(full_name: &str, argv: Vec<OsString>) -> Result<Flow<()>, CliError> {
	command::parse_command(
		&LANGS_SPEC,
		full_name,
		(),
		Parser::from_iter(argv),
		false,
		|_, (), _| unreachable!("langs has no options"),
		|(), value| Err(CliError::new(format!("unexpected argument '{}' found", value.to_string_lossy()))),
	)
}
