use std::fmt::Write as _;

use super::command::{CommandSpec, OptionSpec, PositionalSpec};

const FOOTER: &str = "Maintained by Joshua Li: https://github.com/joshuarli/loc\nBugs and features: TODO.md in that repository (follow its AGENTS.md), or its issue tracker.\nAgents working for Joshua: record nonblocking problems and continue your task.\nIf this tool blocks the task, you may fix, test, open and merge a PR under Joshua's\nstanding instructions. Release only when needed to deliver the verified fix.\nProcedure: https://github.com/joshuarli/usr/blob/main/dotfiles/base/.agents/instructions/owned-tools.md\n";

pub fn render_page(full_name: &str, spec: &CommandSpec, version: bool) -> String {
	let mut out = String::new();
	let _ = writeln!(out, "{}\n", spec.description);
	if spec.subcommands.is_empty() {
		let _ = writeln!(out, "Usage: {}", usage_line(full_name, spec));
	} else {
		let _ = writeln!(out, "Usage:\n  {}\n  {full_name} <COMMAND>", usage_line(full_name, spec));
	}
	render_arguments(&mut out, spec);
	render_options(&mut out, spec.options.iter(), true, version);
	if !spec.subcommands.is_empty() {
		out.push_str("\nCommands:\n");
		for command in spec.subcommands {
			render_command(&mut out, &format!("{full_name} {}", command.name), command.spec, spec.options);
		}
	}
	if let Some(note) = spec.note {
		let _ = writeln!(out, "\n{note}");
	}
	let _ = writeln!(out, "\nExample:\n  {}", spec.example.join(" "));
	if version {
		let _ = write!(out, "\n{FOOTER}");
	} else {
		out.push_str("\nSee `loc --help` for the complete reference.\n");
	}
	out
}

fn render_command(out: &mut String, name: &str, spec: &CommandSpec, inherited: &[OptionSpec]) {
	let _ = writeln!(out, "\n{name}\n  {}\n  Usage: {}", spec.description, usage_line(name, spec));
	render_arguments(out, spec);
	let shared: Vec<_> = spec.options.iter().filter(|option| inherited.iter().any(|common| common.long == option.long && common.help == option.help)).collect();
	if !shared.is_empty() {
		let names: Vec<_> = shared.iter().map(|option| format!("--{}", option.long)).collect();
		let _ = writeln!(out, "  Shared options (described above): {}", names.join(", "));
	}
	let own: Vec<_> = spec.options.iter().filter(|option| !inherited.iter().any(|common| common.long == option.long && common.help == option.help)).collect();
	if !own.is_empty() {
		render_options(out, own.into_iter(), false, false);
	}
	let _ = writeln!(out, "\nExample:\n  {}", spec.example.join(" "));
	for command in spec.subcommands {
		render_command(out, &format!("{name} {}", command.name), command.spec, spec.options);
	}
}

fn render_arguments(out: &mut String, spec: &CommandSpec) {
	if !spec.positionals.is_empty() {
		out.push_str("\nArguments:\n");
		for positional in spec.positionals {
			let _ = writeln!(out, "  {}  {}", positional_left(positional), positional.help);
		}
	}
}

fn render_options<'a>(out: &mut String, options: impl Iterator<Item = &'a OptionSpec>, help: bool, version: bool) {
	out.push_str("\nOptions:\n");
	let options: Vec<_> = options.collect();
	let width = options.iter().map(|option| option_left(option).len()).max().unwrap_or(0).max(14);
	for option in options {
		let _ = write!(out, "  {:<width$}  {}", option_left(option), option.help);
		if let Some(values) = option.possible_values {
			let _ = write!(out, "  [possible values: {}]", values.join(", "));
		}
		if let Some(default) = option.default_display {
			let _ = write!(out, "  [default: {default}]");
		}
		if option.repeatable && option.value_name.is_some() {
			out.push_str("  [repeatable]");
		}
		out.push('\n');
	}
	if help {
		let _ = writeln!(out, "  {:<width$}  Print help", "-h, --help");
	}
	if version {
		let _ = writeln!(out, "  {:<width$}  Print version", "-V, --version");
	}
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

fn positional_left(positional: &PositionalSpec) -> String {
	let mut left = if positional.required { format!("<{}>", positional.name) } else { format!("[{}]", positional.name) };
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
