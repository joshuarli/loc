use std::fmt::Write as _;

use super::command::{CommandSpec, OptionSpec, PositionalSpec, SubcommandSpec};

pub fn render_page(full_name: &str, spec: &CommandSpec, version: bool) -> String {
	let mut out = String::with_capacity(1024);
	let _ = writeln!(out, "{}", spec.description);
	out.push('\n');
	let usage = usage_line(full_name, spec);
	if spec.subcommands.is_empty() {
		let _ = writeln!(out, "Usage: {usage}");
	} else {
		out.push_str("Usage:\n");
		let _ = writeln!(out, "  {usage}");
		let _ = writeln!(out, "  {full_name} <COMMAND>");
	}
	if !spec.positionals.is_empty() {
		out.push_str("\nArguments:\n");
		let lines: Vec<String> = spec.positionals.iter().map(positional_left).collect();
		let width = lines.iter().map(String::len).max().unwrap_or(0);
		for (positional, left) in spec.positionals.iter().zip(&lines) {
			let _ = writeln!(out, "  {left:<width$}  {}", positional.help);
		}
	}
	out.push_str("\nOptions:\n");
	let lefts: Vec<String> = spec.options.iter().map(option_left).collect();
	let help_flag = "-h, --help";
	let version_flag = "-V, --version";
	let width = lefts.iter().map(String::len).max().unwrap_or(0).max(help_flag.len());
	for (option, left) in spec.options.iter().zip(&lefts) {
		let mut line = format!("  {left:<width$}  {}", option.help);
		if let Some(possible_values) = option.possible_values {
			let _ = write!(line, "  [possible values: {}]", possible_values.join(", "));
		}
		if let Some(default) = option.default_display {
			let _ = write!(line, "  [default: {default}]");
		}
		out.push_str(&line);
		out.push('\n');
	}
	let _ = writeln!(out, "  {help_flag:<width$}  Print help");
	if version {
		let _ = writeln!(out, "  {version_flag:<width$}  Print version");
	}
	if !spec.subcommands.is_empty() {
		out.push_str("\nCommands:\n");
		let command_lefts: Vec<String> = spec.subcommands.iter().map(command_left).collect();
		let command_width = command_lefts.iter().map(String::len).max().unwrap_or(0);
		for (subcommand, left) in spec.subcommands.iter().zip(&command_lefts) {
			let _ = writeln!(out, "  {left:<command_width$}  {}", subcommand.about);
		}
	}
	if let Some(note) = spec.note {
		let _ = writeln!(out, "\n{note}");
	}
	out
}

fn usage_line(full_name: &str, spec: &CommandSpec) -> String {
	let mut usage = String::from(full_name);
	if !spec.options.is_empty() {
		usage.push_str(" [OPTIONS]");
	}
	for positional in spec.positionals {
		usage.push(' ');
		usage.push_str(&positional_left(positional));
	}
	usage
}

fn usage_args(spec: &CommandSpec) -> String {
	let mut usage = String::new();
	if !spec.options.is_empty() {
		usage.push_str("[OPTIONS]");
	}
	for positional in spec.positionals {
		if !usage.is_empty() {
			usage.push(' ');
		}
		usage.push_str(&positional_left(positional));
	}
	usage
}

fn positional_left(positional: &PositionalSpec) -> String {
	let brackets = |name: &str| if positional.required { format!("<{name}>") } else { format!("[{name}]") };
	let mut left = brackets(positional.name);
	if positional.repeatable {
		left.push_str("...");
	}
	left
}

fn option_left(option: &OptionSpec) -> String {
	let mut left = option.flag_spelling();
	if let Some(value_name) = option.value_name {
		let _ = write!(left, " <{value_name}>");
	}
	left
}

fn command_left(subcommand: &SubcommandSpec) -> String {
	let args = usage_args(subcommand.spec);
	if args.is_empty() { subcommand.name.to_owned() } else { format!("{} {args}", subcommand.name) }
}
