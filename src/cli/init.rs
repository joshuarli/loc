use std::{
	ffi::OsString,
	path::PathBuf,
	process::ExitCode,
};

use lexopt::Parser;

use super::{
	command::{self, CommandSpec, Flow, OptionSpec},
	error::CliError,
};
use crate::init;

pub const OUTPUT_ID: &str = "output";
pub const FORCE_ID: &str = "force";

const OUTPUT: OptionSpec = OptionSpec::value(OUTPUT_ID, Some('o'), "output", "OUTPUT", None, Some(".loc.toml"), false, "Output path for the configuration file");
const FORCE: OptionSpec = OptionSpec::flag(FORCE_ID, Some('f'), "force", "Overwrite the file if it already exists");

pub static INIT_SPEC: CommandSpec = CommandSpec {
	description: "Generate a default configuration file",
	example: &["loc", "init"],
	options: &[OUTPUT, FORCE],
	positionals: &[],
	subcommands: &[],
	conflicts: &[],
	note: None,
};

pub fn run(full_name: &str, argv: Vec<OsString>) -> ExitCode {
	super::exit_with(parse(full_name, argv), |args, _| {
		init::run_init(args.output, args.force)
	})
}

#[derive(Default)]
pub(super) struct InitArgs {
	output: Option<PathBuf>,
	force: bool,
}

fn apply_option(option: &'static OptionSpec, args: &mut InitArgs, raw: Option<OsString>) -> Result<(), CliError> {
	match option.id {
		OUTPUT_ID => {
			args.output = Some(PathBuf::from(raw.ok_or_else(|| CliError::new("internal error: option '--output' is not a flag"))?));
		}
		FORCE_ID => args.force = true,
		_ => return Err(CliError::new(format!("internal error: unexpected option --{}", option.long))),
	}
	Ok(())
}

pub(super) fn parse(full_name: &str, argv: Vec<OsString>) -> Result<Flow<InitArgs>, CliError> {
	command::parse_command(
		&INIT_SPEC,
		full_name,
		InitArgs::default(),
		Parser::from_iter(argv),
		false,
		apply_option,
		|_, value| Err(CliError::new(format!("unexpected argument '{}' found", value.to_string_lossy()))),
	)
}
