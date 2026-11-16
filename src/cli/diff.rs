use std::{
	collections::HashSet,
	io::{self, Write as _},
	path::PathBuf,
};

use anyhow::{Context as _, bail, ensure};
use anyhow::Result;
use lexopt::Parser;

use super::{
	analyze::{self, PATH_POSITIONAL, SourceArgs, FormatArgs, ids},
	command::{self, CommandSpec, Flow, OptionSpec, PositionalSpec},
	error::CliError,
};
use crate::{
	analysis::{Snapshot, count_snapshot},
	display::Versions,
};

pub const TO_ID: &str = "to";
pub const TO: OptionSpec = OptionSpec::value(TO_ID, None, "to", "REV", None, None, false, "The revision to compare to; defaults to the working tree");

const FROM_POSITIONAL: PositionalSpec = PositionalSpec {
	name: "FROM",
	required: true,
	repeatable: false,
	help: "The revision to compare from: a commit, tag or branch",
};

pub static DIFF_SPEC: CommandSpec = CommandSpec {
	description: "Compare line counts between two versions of a git repository, by language",
	example: &["loc", "diff", "HEAD"],
	options: &[
		analyze::CONFIG,
		analyze::NO_CONFIG,
		analyze::INCLUDE_GENERATED,
		analyze::MAX_DEPTH,
		analyze::HIDDEN,
		analyze::EXCLUDE,
		analyze::INCLUDE_LANG,
		analyze::EXCLUDE_LANG,
		analyze::EXT,
		analyze::FAIL_ON_ERROR,
		analyze::NUMBER_STYLE,
		analyze::SIZE_STYLE,
		analyze::INDENT,
		analyze::OUTPUT,
		analyze::TOP_LANGUAGES,
		TO,
	],
	positionals: &[FROM_POSITIONAL, PATH_POSITIONAL],
	subcommands: &[],
	conflicts: &[(ids::NO_CONFIG, ids::CONFIG), (ids::INCLUDE_LANG, ids::EXCLUDE_LANG)],
	note: Some("These options are shared with the main command."),
};

/// Arguments for `loc diff`.
pub struct DiffArgs {
	pub from: String,
	pub to: Option<String>,
	pub source: SourceArgs,
	pub format: FormatArgs,
}

#[derive(Default)]
struct ScratchArgs {
	from: Option<String>,
	to: Option<String>,
	source: SourceArgs,
	format: FormatArgs,
}

fn apply_option(option: &'static OptionSpec, args: &mut ScratchArgs, raw: Option<std::ffi::OsString>) -> Result<(), CliError> {
	if option.id == TO_ID {
		let value = command::utf8_value(option, raw.ok_or_else(|| CliError::new("internal error: option '--to' is not a flag"))?.as_os_str())?;
		args.to = Some(value);
		return Ok(());
	}
	analyze::apply_shared(option, &mut args.source, &mut args.format, raw)
}

fn apply_positional(args: &mut ScratchArgs, value: std::ffi::OsString) -> Result<(), CliError> {
	if args.from.is_none() {
		args.from = Some(
			value
				.into_string()
				.map_err(|_| CliError::new("the <FROM> revision requires a value in valid UTF-8"))?,
		);
		return Ok(());
	}
	args.source.paths.push(PathBuf::from(value));
	Ok(())
}

fn into_args(scratch: ScratchArgs) -> Result<DiffArgs, CliError> {
	let from = scratch.from.ok_or_else(|| CliError::new("missing required argument <FROM>"))?;
	Ok(DiffArgs { from, to: scratch.to, source: scratch.source, format: scratch.format })
}

pub(super) fn run(full_name: &str, argv: Vec<std::ffi::OsString>) -> std::process::ExitCode {
	super::exit_with(parse(full_name, argv), |args, seen| execute(&args, &seen))
}

pub(super) fn parse(full_name: &str, argv: Vec<std::ffi::OsString>) -> Result<Flow<DiffArgs>, CliError> {
	let flow = command::parse_command(
		&DIFF_SPEC,
		full_name,
		ScratchArgs::default(),
		Parser::from_iter(argv),
		false,
		apply_option,
		apply_positional,
	)?;
	match flow {
		Flow::Done(scratch, seen) => Ok(Flow::Done(into_args(scratch)?, seen)),
		Flow::Help(page) => Ok(Flow::Help(page)),
		Flow::Version(version) => Ok(Flow::Version(version)),
	}
}

pub fn execute(args: &DiffArgs, seen: &HashSet<&'static str>) -> Result<()> {
	let config = super::merge::merge_diff_config(super::merge::base_config(&args.source)?, args, seen)?;
	let [path] = &config.paths[..] else {
		bail!("`loc diff` compares one path at a time");
	};
	ensure!(path.as_os_str() != "-", "`loc diff` can't read its paths from standard input");
	super::merge::ensure_path(&config)?;
	let from = count_snapshot(path, &Snapshot::Revision(&args.from), &config.analysis, false)
		.with_context(|| format!("couldn't read revision `{}`", args.from))?;
	let to = match &args.to {
		Some(revision) => count_snapshot(path, &Snapshot::Revision(revision), &config.analysis, false)
			.with_context(|| format!("couldn't read revision `{revision}`"))?,
		// The working tree's submodules are left out, as a revision's can't be read.
		None => count_snapshot(path, &Snapshot::WorkingTree { submodules: false }, &config.analysis, false)?,
	};
	let versions = Versions { from_name: &args.from, to_name: args.to.as_deref(), from: &from, to: &to };
	let mut stdout = io::BufWriter::new(io::stdout().lock());
	crate::display::write_diff(config.display.output, &versions, path, (&config).into(), &mut stdout)?;
	stdout.flush()?;
	let skipped = from.skipped_entries() + to.skipped_entries();
	if config.analysis.fail_on_error && skipped > 0 {
		return Err(anyhow::anyhow!("Skipped {skipped} {} due to errors", crate::display::formatting::pluralize(skipped, "entry", "entries")));
	}
	Ok(())
}


