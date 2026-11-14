use std::{
	collections::HashSet,
	ffi::{OsStr, OsString},
	fmt::Write as _,
	process::ExitCode,
};

use lexopt::{Arg, Parser};

use super::{error::CliError, help};

#[derive(Clone, Copy)]
pub struct OptionSpec {
	pub id: &'static str,
	pub short: Option<char>,
	pub long: &'static str,
	pub value_name: Option<&'static str>,
	pub possible_values: Option<&'static [&'static str]>,
	pub default_display: Option<&'static str>,
	pub repeatable: bool,
	pub help: &'static str,
}

impl OptionSpec {
	pub const fn flag(id: &'static str, short: Option<char>, long: &'static str, help: &'static str) -> Self {
		Self { id, short, long, value_name: None, possible_values: None, default_display: None, repeatable: true, help }
	}

	#[expect(clippy::too_many_arguments, reason = "the const constructor mirrors the fields it sets")]
	pub const fn value(
		id: &'static str,
		short: Option<char>,
		long: &'static str,
		value_name: &'static str,
		possible_values: Option<&'static [&'static str]>,
		default_display: Option<&'static str>,
		repeatable: bool,
		help: &'static str,
	) -> Self {
		Self { id, short, long, value_name: Some(value_name), possible_values, default_display, repeatable, help }
	}

	pub fn spelling(&self) -> String {
		self.value_name.map_or_else(|| format!("--{}", self.long), |value_name| format!("--{} <{value_name}>", self.long))
	}

	pub fn flag_spelling(&self) -> String {
		self.short.map_or_else(|| format!("    --{}", self.long), |short| format!("-{short}, --{}", self.long))
	}
}

pub struct PositionalSpec {
	pub name: &'static str,
	pub required: bool,
	pub repeatable: bool,
	pub help: &'static str,
}

pub struct SubcommandSpec {
	pub name: &'static str,
	pub about: &'static str,
	pub spec: &'static CommandSpec,
	pub run: fn(&str, Vec<OsString>) -> ExitCode,
}

pub struct CommandSpec {
	pub description: &'static str,
	pub options: &'static [OptionSpec],
	pub positionals: &'static [PositionalSpec],
	pub subcommands: &'static [SubcommandSpec],
	pub conflicts: &'static [(&'static str, &'static str)],
	pub note: Option<&'static str>,
}

impl CommandSpec {
	pub fn find_long(&self, long: &str) -> Option<&'static OptionSpec> {
		self.options.iter().find(|option| option.long == long)
	}

	pub fn find_short(&self, short: char) -> Option<&'static OptionSpec> {
		self.options.iter().find(|option| option.short == Some(short))
	}

	pub fn find(&self, id: &str) -> Option<&'static OptionSpec> {
		self.options.iter().find(|option| option.id == id)
	}
}

pub enum Flow<A> {
	Done(A, HashSet<&'static str>),
	Help(String),
	Version(String),
}

pub fn parse_command<A>(
	spec: &'static CommandSpec,
	full_name: &str,
	mut args: A,
	parser: Parser,
	version_allowed: bool,
	mut apply_option: impl FnMut(&'static OptionSpec, &mut A, Option<OsString>) -> Result<(), CliError>,
	mut apply_positional: impl FnMut(&mut A, OsString) -> Result<(), CliError>,
) -> Result<Flow<A>, CliError> {
	let mut parser = parser;
	let mut seen: HashSet<&'static str> = HashSet::new();
	while let Some(arg) = parser.next()? {
		match arg {
			Arg::Value(value) => apply_positional(&mut args, value)?,
			Arg::Short('h') | Arg::Long("help") => return Ok(Flow::Help(help::render_page(full_name, spec, version_allowed))),
			Arg::Short('V') | Arg::Long("version") if version_allowed => {
				return Ok(Flow::Version(super::version::version_line()));
			}
			Arg::Short(short) => {
				let option = spec
					.find_short(short)
					.ok_or_else(|| CliError::unknown_option(&format!("-{short}")))?;
				apply(option, &mut parser, &mut args, &mut seen, &mut apply_option)?;
			}
			Arg::Long(long) => {
				let option = spec
					.find_long(long)
					.ok_or_else(|| CliError::unknown_option(&format!("--{long}")))?;
				apply(option, &mut parser, &mut args, &mut seen, &mut apply_option)?;
			}
		}
	}
	for &(first, second) in spec.conflicts {
		if seen.contains(first) && seen.contains(second) {
			let first = spec.find(first).map_or_else(|| format!("--{first}"), OptionSpec::spelling);
			let second = spec.find(second).map_or_else(|| format!("--{second}"), OptionSpec::spelling);
			return Err(CliError::new(format!("the argument '{first}' cannot be used with '{second}'")));
		}
	}
	Ok(Flow::Done(args, seen))
}

fn apply<A>(
	option: &'static OptionSpec,
	parser: &mut Parser,
	args: &mut A,
	seen: &mut HashSet<&'static str>,
	apply_option: &mut impl FnMut(&'static OptionSpec, &mut A, Option<OsString>) -> Result<(), CliError>,
) -> Result<(), CliError> {
	let raw = match option.value_name {
		Some(_) => {
			if !option.repeatable && seen.contains(option.id) {
				return Err(CliError::new(format!(
					"the argument '{}' was provided more than once, but cannot be used multiple times",
					option.spelling()
				)));
			}
			let raw = parser.value()?;
			if let Some(possible) = option.possible_values {
				let value = utf8_value(option, &raw)?;
				if !possible.contains(&value.as_str()) {
					return Err(invalid_value(option, &value, possible));
				}
			}
			Some(raw)
		}
		None => None,
	};
	apply_option(option, args, raw)?;
	seen.insert(option.id);
	Ok(())
}

pub fn utf8_value(option: &OptionSpec, raw: &OsStr) -> Result<String, CliError> {
	raw.to_str().map(str::to_owned).ok_or_else(|| {
		CliError::new(format!("option '{}' requires a value in valid UTF-8", option.spelling()))
	})
}

fn invalid_value_spelling(spelling: &str, value: &str, possible: &[&'static str]) -> CliError {
	let possible_list = possible.join(", ");
	let mut message = format!("invalid value '{value}' for '{spelling}'\n  [possible values: {possible_list}]");
	if let Some(suggestion) = suggest(value, possible) {
		let _ = write!(message, "\n  did you mean '{suggestion}'?");
	}
	CliError::new(message)
}

fn invalid_value(option: &OptionSpec, value: &str, possible: &[&'static str]) -> CliError {
	invalid_value_spelling(&option.spelling(), value, possible)
}

fn suggest<'a>(value: &str, candidates: &[&'a str]) -> Option<&'a str> {
	candidates
		.iter()
		.map(|candidate| (levenshtein(value, candidate), *candidate))
		.filter(|(distance, _)| *distance <= 2)
		.min_by_key(|(distance, _)| *distance)
		.map(|(_, candidate)| candidate)
}

fn levenshtein(a: &str, b: &str) -> usize {
	let a: Vec<char> = a.chars().collect();
	let b: Vec<char> = b.chars().collect();
	let mut row: Vec<usize> = (0..=b.len()).collect();
	for (i, a_char) in a.iter().enumerate() {
		let mut previous = row[0];
		row[0] = i + 1;
		for (j, b_char) in b.iter().enumerate() {
			let current = row[j + 1];
			row[j + 1] = if a_char == b_char {
				previous
			} else {
				(previous + 1).min(row[j] + 1).min(current + 1)
			};
			previous = current;
		}
	}
	row[b.len()]
}

#[cfg(test)]
mod tests {
	use super::*;

	const ALPHA: OptionSpec = OptionSpec::flag("alpha", Some('a'), "alpha", "alpha help");
	const BETA: OptionSpec = OptionSpec::value("beta", Some('b'), "beta", "BETA", None, None, false, "beta help");
	const GAMMA: OptionSpec =
		OptionSpec::value("gamma", None, "gamma", "GAMMA", Some(&["one", "two"]), None, false, "gamma help");
	const REPEATABLE: OptionSpec = OptionSpec::value("repeatable", Some('r'), "repeatable", "VALUE", None, None, true, "repeatable help");

	const SPEC: CommandSpec = CommandSpec {
		description: "test command",
		options: &[ALPHA, BETA, GAMMA, REPEATABLE],
		positionals: &[],
		subcommands: &[],
		conflicts: &[("alpha", "beta")],
		note: None,
	};

	#[derive(Default)]
	struct Args {
		alpha: bool,
		beta: Option<String>,
		gamma: Option<String>,
		repeatable: Vec<String>,
		positional: Vec<OsString>,
	}

	fn test_apply(option: &'static OptionSpec, args: &mut Args, raw: Option<OsString>) -> Result<(), CliError> {
		match option.id {
			"alpha" => args.alpha = true,
			"beta" => args.beta = Some(utf8_value(option, &raw.expect("value"))?),
			"gamma" => args.gamma = Some(utf8_value(option, &raw.expect("value"))?),
			"repeatable" => args.repeatable.push(utf8_value(option, &raw.expect("value"))?),
			_ => unreachable!(),
		}
		Ok(())
	}

	#[expect(clippy::unnecessary_wraps, reason = "positional callbacks share a fallible signature across commands")]
	fn positional(args: &mut Args, value: OsString) -> Result<(), CliError> {
		args.positional.push(value);
		Ok(())
	}

	fn parse(argv: &[&str]) -> Result<Flow<Args>, CliError> {
		let argv = argv.iter().map(OsString::from).collect::<Vec<_>>();
		parse_command(&SPEC, "loc", Args::default(), Parser::from_iter(argv), false, test_apply, positional)
	}

	fn done(result: Result<Flow<Args>, CliError>) -> (Args, HashSet<&'static str>) {
		match result {
			Ok(Flow::Done(args, seen)) => (args, seen),
			Ok(Flow::Help(_)) => panic!("expected done, got help"),
			Ok(Flow::Version(_)) => panic!("expected done, got version"),
			Err(error) => panic!("expected done, got error: {error}"),
		}
	}

	#[test]
	fn flags_and_clusters_and_equals_are_parsed() {
		let (args, seen) = done(parse(&["loc", "-a"]));
		assert!(args.alpha);
		assert!(seen.contains("alpha"));
		let (args, _) = done(parse(&["loc", "-ar", "4"]));
		assert!(args.alpha && args.repeatable == ["4"]);
		let (args, _) = done(parse(&["loc", "-b=5"]));
		assert_eq!(args.beta.as_deref(), Some("5"));
		let (args, _) = done(parse(&["loc", "--beta", "6"]));
		assert_eq!(args.beta.as_deref(), Some("6"));
		let (args, _) = done(parse(&["loc", "--beta=7"]));
		assert_eq!(args.beta.as_deref(), Some("7"));
	}

	#[test]
	fn unknown_option_is_an_error() {
		let error = parse(&["loc", "--nope"]).err().expect("unknown option");
		assert_eq!(error.to_string(), "unexpected argument '--nope' found");
	}

	#[test]
	fn missing_value_is_an_error() {
		let error = parse(&["loc", "--beta"]).err().expect("missing value");
		assert!(error.to_string().contains("missing argument for option"), "{error}");
	}

	#[test]
	fn invalid_possible_value_lists_values_and_suggests() {
		let error = parse(&["loc", "--gamma", "onne"]).err().expect("invalid value");
		assert!(error.to_string().contains("'--gamma <GAMMA>'"), "{error}");
		assert!(error.to_string().contains("possible values: one, two"), "{error}");
		assert!(error.to_string().contains("did you mean 'one'?"), "{error}");
		let (args, _) = done(parse(&["loc", "--gamma", "one"]));
		assert_eq!(args.gamma.as_deref(), Some("one"));
	}

	#[test]
	fn non_repeatable_value_may_not_be_repeated() {
		let error = parse(&["loc", "--beta", "1", "--beta", "2"]).err().expect("repeated value");
		assert!(error.to_string().contains("more than once"), "{error}");
		let (args, _) = done(parse(&["loc", "--repeatable", "1", "--repeatable", "2"]));
		assert_eq!(args.repeatable, ["1", "2"]);
	}

	#[test]
	fn conflicting_options_are_an_error() {
		let error = parse(&["loc", "-a", "-b", "1"]).err().expect("conflict");
		assert_eq!(error.to_string(), "the argument '--alpha' cannot be used with '--beta <BETA>'");
	}

	#[test]
	fn everything_after_double_dash_is_positional() {
		let (args, _) = done(parse(&["loc", "--", "-a"]));
		assert_eq!(args.positional, [OsString::from("-a")]);
		assert!(!args.alpha);
	}

	#[test]
	fn help_is_an_early_exit() {
		assert!(matches!(parse(&["loc", "--help"]), Ok(Flow::Help(_))));
		assert!(matches!(parse(&["loc", "-h"]), Ok(Flow::Help(_))));
	}

	#[cfg(unix)]
	#[test]
	fn non_utf8_value_is_an_error() {
		use std::os::unix::ffi::OsStringExt;
		let argv = [OsString::from("loc"), OsString::from("--beta"), OsString::from_vec(vec![0xff])];
		let result = parse_command(&SPEC, "loc", Args::default(), Parser::from_iter(argv), false, test_apply, positional);
		assert!(result.is_err());
	}
}
