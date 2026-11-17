mod analyze;
mod command;
mod diff;
mod error;
mod help;
mod init;
mod langs;
mod merge;
mod version;

use std::{
	collections::HashSet,
	ffi::{OsStr, OsString},
	io::{self, Write as _},
	process::ExitCode,
};

use command::{CommandSpec, Flow, SubcommandSpec};
use error::{CliError, is_broken_pipe, report_usage};

pub const BIN: &str = "loc";

/// The specification of the main command, shared with the help renderer and the completion generators.
pub static ROOT_SPEC: CommandSpec = CommandSpec {
	description: "Count code, comment, blank, and shebang lines by language, as text or JSON",
	example: &["loc", "src"],
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
		analyze::QUIET,
		analyze::VERBOSE,
		analyze::NO_GITIGNORE,
		analyze::NO_GITATTRIBUTES,
		analyze::GIT,
		analyze::SYMLINKS,
		analyze::NUMBER_STYLE,
		analyze::SIZE_STYLE,
		analyze::INDENT,
		analyze::OUTPUT,
		analyze::TOP_LANGUAGES,
		analyze::PERCENT_PRECISION,
		analyze::LANGUAGE_SORT,
		analyze::SORT_DIRECTION,
		analyze::MIN_LINES,
		analyze::BY_DIR,
	],
	positionals: &[analyze::PATH_POSITIONAL],
	subcommands: &[
		SubcommandSpec { name: "langs", spec: &langs::LANGS_SPEC, run: langs::run },
		SubcommandSpec {
			name: "diff",
			spec: &diff::DIFF_SPEC,
			run: diff::run,
		},
		SubcommandSpec { name: "init", spec: &init::INIT_SPEC, run: init::run },
	],
	conflicts: &[
		(analyze::ids::QUIET, analyze::ids::VERBOSE),
		(analyze::ids::NO_CONFIG, analyze::ids::CONFIG),
		(analyze::ids::INCLUDE_LANG, analyze::ids::EXCLUDE_LANG),
	],
	note: None,
};

/// Parse `args` (including the program name, as from `std::env::args_os()`) and run the selected command.
pub fn run<I, T>(input: I) -> ExitCode
where
	I: IntoIterator<Item = T>,
	T: Into<OsString>,
{
	let argv: Vec<OsString> = input.into_iter().map(Into::into).collect();
	match select_subcommand(&argv) {
		Selection::Root(argv) => exit_with(analyze::parse_root(BIN, true, argv), |args, seen| analyze::execute(&args, &seen)),
		Selection::Subcommand(command, rest) => {
			let full_name = format!("{BIN} {}", command.name);
			let argv = std::iter::once(OsString::from(&full_name)).chain(rest).collect();
			(command.run)(&full_name, argv)
		}
		Selection::EarlyExit(code) => code,
	}
}

pub fn exit_with<A>(flow: Result<Flow<A>, CliError>, execute: impl FnOnce(A, HashSet<&'static str>) -> anyhow::Result<()>) -> ExitCode {
	match flow {
		Ok(Flow::Done(args, seen)) => finish(execute(args, seen)),
		Ok(Flow::Help(page) | Flow::Version(page)) => print_page(&page),
		Err(error) => {
			report_usage(&error);
			ExitCode::from(2)
		}
	}
}

fn print_page(text: &str) -> ExitCode {
	let mut stdout = io::stdout().lock();
	let result = stdout.write_all(text.as_bytes()).and_then(|()| {
		if text.ends_with('\n') {
			Ok(())
		} else {
			stdout.write_all(b"\n")
		}
	}).and_then(|()| stdout.flush());
	match result {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("loc: couldn't write to stdout: {error}");
			ExitCode::FAILURE
		}
	}
}

fn finish(result: anyhow::Result<()>) -> ExitCode {
	match result {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) if is_broken_pipe(&error) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("Error: {error:#}");
			ExitCode::FAILURE
		}
	}
}

enum Selection {
	Root(Vec<OsString>),
	Subcommand(&'static SubcommandSpec, Vec<OsString>),
	EarlyExit(ExitCode),
}

/// Split `argv` at the first positional token: when it names a registered subcommand, dispatch to it.
/// Options before the subcommand belong to the main command; they parse there and are ignored.
/// Everything after `--` is positional and never a subcommand.
fn select_subcommand(argv: &[OsString]) -> Selection {
	let limit = argv.iter().position(|arg| arg == "--").unwrap_or(argv.len());
	let mut index = 1;
	while index < limit {
		let arg = &argv[index];
		if let Some((name, inline_value)) = long_option(arg) {
			match ROOT_SPEC.find_long(name) {
				Some(option) => index += if inline_value || option.value_name.is_none() { 1 } else { 2 },
				None => return Selection::Root(argv.to_vec()),
			}
			continue;
		}
		if let Some(step) = short_option_step(arg) {
			index += step;
			continue;
		}
		if arg.as_encoded_bytes().starts_with(b"-") && arg.len() > 1 {
			return Selection::Root(argv.to_vec());
		}
		if let Some(command) = ROOT_SPEC.subcommands.iter().find(|command| arg.as_os_str() == OsStr::new(command.name)) {
			match analyze::parse_prefix(&argv[1..index]) {
				Ok(Flow::Done(..)) => return Selection::Subcommand(command, argv[index + 1..].to_vec()),
				Ok(Flow::Help(page) | Flow::Version(page)) => return Selection::EarlyExit(print_page(&page)),
				Err(error) => {
					report_usage(&error);
					return Selection::EarlyExit(ExitCode::from(2));
				}
			}
		}
		return Selection::Root(argv.to_vec());
	}
	Selection::Root(argv.to_vec())
}

fn long_option(arg: &OsStr) -> Option<(&str, bool)> {
	let text = arg.to_str()?;
	let body = text.strip_prefix("--")?;
	if body.is_empty() {
		return None;
	}
	Some(match body.split_once('=') {
		Some((name, _)) => (name, true),
		None => (body, false),
	})
}

fn short_option_step(arg: &OsStr) -> Option<usize> {
	let text = arg.to_str()?;
	let body = text.strip_prefix('-')?;
	if body.is_empty() || body.starts_with('-') {
		return None;
	}
	let count = body.chars().count();
	for (position, short) in body.chars().enumerate() {
		match ROOT_SPEC.find_short(short) {
			Some(option) if option.value_name.is_some() => return Some(if position == count - 1 { 2 } else { 1 }),
			Some(_) => {}
			None => return None,
		}
	}
	Some(1)
}

#[cfg(test)]
mod tests {
	use super::*;

	static INTERNAL_SPECS: &[&CommandSpec] = &[&ROOT_SPEC, &langs::LANGS_SPEC, &diff::DIFF_SPEC, &init::INIT_SPEC];

	fn all_specs() -> Vec<&'static CommandSpec> {
		INTERNAL_SPECS.to_vec()
	}

	#[test]
	fn every_command_and_option_is_explained() {
		for spec in all_specs() {
			assert!(!spec.description.trim().is_empty(), "command has no description");
			for option in spec.options {
				assert!(!option.help.trim().is_empty(), "--{} has no help", option.long);
				assert!(!option.long.is_empty(), "an option has no long name");
			}
			for positional in spec.positionals {
				assert!(!positional.help.trim().is_empty(), "<{}> has no help", positional.name);
			}
		}
	}

	#[test]
	fn option_names_are_unique_per_command() {
		for spec in all_specs() {
			let longs: HashSet<_> = spec.options.iter().map(|option| option.long).collect();
			assert_eq!(longs.len(), spec.options.len(), "duplicate long option");
			let shorts: HashSet<_> = spec.options.iter().filter_map(|option| option.short).collect();
			assert_eq!(shorts.len(), spec.options.iter().filter_map(|option| option.short).count(), "duplicate short option");
		}
	}

	#[test]
	fn conflict_pairs_reference_registered_options() {
		for spec in all_specs() {
			for &(first, second) in spec.conflicts {
				assert!(spec.find(first).is_some(), "conflict references unknown option {first}");
				assert!(spec.find(second).is_some(), "conflict references unknown option {second}");
			}
		}
	}

	#[test]
	fn subcommand_names_are_unique() {
		let names: HashSet<_> = ROOT_SPEC.subcommands.iter().map(|command| command.name).collect();
		assert_eq!(names.len(), ROOT_SPEC.subcommands.len());
	}

	#[test]
	fn diff_shares_the_source_and_format_options_in_order() {
		let diff: HashSet<_> = diff::DIFF_SPEC.options.iter().map(|option| option.id).collect();
		for option in analyze::SOURCE_OPTIONS.iter().chain(analyze::FORMAT_OPTIONS) {
			assert!(diff.contains(option.id), "diff is missing source/format option --{}", option.long);
		}
	}

	#[test]
	fn schema_examples_parse_without_running_commands() {
		let example_argv = |spec: &CommandSpec| spec.example.iter().map(OsString::from).collect::<Vec<_>>();
		assert!(matches!(analyze::parse_root(BIN, true, example_argv(&ROOT_SPEC)), Ok(Flow::Done(..))));
		for command in ROOT_SPEC.subcommands {
			assert_eq!(command.spec.example[..2], [BIN, command.name]);
			let args = std::iter::once(OsString::from(format!("loc {}", command.name)))
				.chain(command.spec.example.iter().skip(2).map(OsString::from)).collect();
			let parsed = match command.name {
				"diff" => matches!(diff::parse("loc diff", args), Ok(Flow::Done(..))),
				"init" => matches!(init::parse("loc init", args), Ok(Flow::Done(..))),
				"langs" => matches!(langs::parse("loc langs", args), Ok(Flow::Done(..))),
				name => panic!("add parser coverage for {name}"),
			};
			assert!(parsed, "invalid example: {}", command.spec.example.join(" "));
		}
	}

	#[test]
	fn root_help_lists_every_command() {
		let page = help::render_page(BIN, &ROOT_SPEC, true);
		for spec in all_specs() {
			for option in spec.options {
				assert!(page.contains(&format!("--{}", option.long)), "root help omits --{}", option.long);
			}
			for positional in spec.positionals {
				assert!(page.contains(positional.help), "root help omits {}", positional.name);
			}
			assert!(page.contains(&spec.example.join(" ")));
		}
		for command in ROOT_SPEC.subcommands {
			assert!(page.contains(command.spec.description));
		}
	}
}
