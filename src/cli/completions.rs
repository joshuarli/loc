use std::{
	ffi::OsString,
	fmt::Write as _,
	io::{self, Write as _},
	process::ExitCode,
};

use lexopt::Parser;

use super::{
	ROOT_SPEC,
	command::{self, CommandSpec, Flow, PositionalSpec},
	error::CliError,
};
use crate::display::{LanguageSortKey, NumberStyle, OutputFormat, SizeStyle, SortDirection};

pub enum Shell {
	Bash,
	Zsh,
	Fish,
	Powershell,
	Elvish,
}

impl Shell {
	pub const VALUES: &'static [&'static str] = &["bash", "zsh", "fish", "powershell", "elvish"];

	pub fn from_cli(value: &str) -> Option<Self> {
		match value {
			"bash" => Some(Self::Bash),
			"zsh" => Some(Self::Zsh),
			"fish" => Some(Self::Fish),
			"powershell" => Some(Self::Powershell),
			"elvish" => Some(Self::Elvish),
			_ => None,
		}
	}

	fn render(&self) -> String {
		match self {
			Self::Bash => render_bash(),
			Self::Zsh => render_zsh(),
			Self::Fish => render_fish(),
			Self::Powershell => render_powershell(),
			Self::Elvish => render_elvish(),
		}
	}
}

const SHELL_POSITIONAL: PositionalSpec = PositionalSpec {
	name: "SHELL",
	required: true,
	repeatable: false,
	help: "The shell to generate completions for (bash, zsh, fish, powershell, elvish)",
};

pub static COMPLETIONS_SPEC: CommandSpec = CommandSpec {
	description: "Generate shell completion scripts",
	options: &[],
	positionals: &[SHELL_POSITIONAL],
	subcommands: &[],
	conflicts: &[],
	note: None,
};

pub fn run(full_name: &str, argv: Vec<OsString>) -> ExitCode {
	super::exit_with(parse(full_name, argv), |args, _| {
		let script = args.shell.render();
		let mut stdout = io::BufWriter::new(io::stdout().lock());
		stdout.write_all(script.as_bytes())?;
		stdout.flush()?;
		Ok(())
	})
}

struct CompletionsArgs {
	shell: Shell,
}

fn parse(full_name: &str, argv: Vec<OsString>) -> Result<Flow<CompletionsArgs>, CliError> {
	let flow = command::parse_command(
		&COMPLETIONS_SPEC,
		full_name,
		None,
		Parser::from_iter(argv),
		false,
		|_, _, _| Err(CliError::new("internal error: completions has no options")),
		apply_positional,
	)?;
	match flow {
		Flow::Done(shell, seen) => {
			let shell = shell.ok_or_else(|| CliError::new("missing required argument <SHELL>"))?;
			Ok(Flow::Done(CompletionsArgs { shell }, seen))
		}
		Flow::Help(page) => Ok(Flow::Help(page)),
		Flow::Version(version) => Ok(Flow::Version(version)),
	}
}

type ShellSlot = Option<Shell>;

fn apply_positional(slot: &mut ShellSlot, value: OsString) -> Result<(), CliError> {
	let value = into_utf8(value)?;
	let shell = Shell::from_cli(&value)
		.ok_or_else(|| invalid_value("<SHELL>", &value, Shell::VALUES))?;
	*slot = Some(shell);
	Ok(())
}

fn into_utf8(value: OsString) -> Result<String, CliError> {
	value.into_string().map_err(|_| CliError::new("the <SHELL> argument requires a value in valid UTF-8"))
}

fn invalid_value(spelling: &str, value: &str, possible: &[&'static str]) -> CliError {
	command::invalid_value_spelling(spelling, value, possible)
}

fn longs(spec: &CommandSpec) -> String {
	spec.options.iter().map(|option| format!("--{}", option.long)).collect::<Vec<_>>().join(" ")
}

fn shorts(spec: &CommandSpec) -> String {
	spec.options.iter().filter_map(|option| option.short.map(|short| format!("-{short}"))).collect::<Vec<_>>().join(" ")
}

fn longs_and_shorts(spec: &CommandSpec) -> String {
	match (shorts(spec), longs(spec)) {
		(s, l) if s.is_empty() => l,
		(s, l) => format!("{s} {l}"),
	}
}

fn subcommand_names() -> String {
	ROOT_SPEC.subcommands.iter().map(|subcommand| subcommand.name).collect::<Vec<_>>().join(" ")
}

fn subcommand_spec(name: &str) -> &'static CommandSpec {
	ROOT_SPEC
		.subcommands
		.iter()
		.find(|subcommand| subcommand.name == name)
		.map(|subcommand| subcommand.spec)
		.expect("registered subcommand")
}

const ENUM_CHOICES: &[(&str, &[&str])] = &[
	("output", OutputFormat::VALUES),
	("number-style", NumberStyle::VALUES),
	("size-units", SizeStyle::VALUES),
	("sort-by", LanguageSortKey::VALUES),
	("sort-direction", SortDirection::VALUES),
];

fn render_bash() -> String {
	let root = longs_and_shorts(&ROOT_SPEC);
	let mut script = format!(
		"# cs completions for bash\n_cs() {{\n\tlocal cur prev cmd\n\tcur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n\tprev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n\tcmd=\"\"\n\tfor ((i = 1; i < COMP_CWORD; i++)); do\n\t\tcase \"${{COMP_WORDS[i]}}\" in\n\t\t\t{0}) cmd=\"${{COMP_WORDS[i]}}\"; break ;;\n\t\t\t-*) ;;\n\t\t\t*) break ;;\n\t\tesac\n\tdone\n\tlocal words\n\tcase \"$prev\" in\n",
		subcommand_names().replace(' ', "|")
	);
	for (option, values) in ENUM_CHOICES {
		if let Some(short) = option_short(option) {
			let _ = writeln!(script, "\t\t-{short}|--{option}) words=\"{}\" ;;", values.join(" "));
		} else {
			let _ = writeln!(script, "\t\t--{option}) words=\"{}\" ;;", values.join(" "));
		}
	}
	let _ = writeln!(script, "\t*)\n\t\tcase \"$cmd\" in\n\t\t\t\"\") words=\"{} {}\" ;;", subcommand_names(), root);
	let _ = writeln!(script, "\t\t\tcompletions) words=\"{}\" ;;", Shell::VALUES.join(" "));
	let _ = writeln!(script, "\t\t\tlangs) words=\"\" ;;");
	let _ = writeln!(script, "\t\t\tdiff) words=\"{}\" ;;", longs_and_shorts(subcommand_spec("diff")));
	let _ = writeln!(script, "\t\t\tinit) words=\"{}\" ;;", longs_and_shorts(subcommand_spec("init")));
	let _ = writeln!(script, "\t\tesac\n\t;;\n\tesac\n\tCOMPREPLY=( $(compgen -W \"$words\" -- \"$cur\") )\n}}\ncomplete -F _cs cs\n");
	script
}

fn option_short(option: &str) -> Option<char> {
	ROOT_SPEC.find_long(option).and_then(|spec| spec.short)
}

fn render_zsh() -> String {
	let root = longs_and_shorts(&ROOT_SPEC);
	let mut script = String::from("#compdef cs\n_cs() {\n\tlocal -a commands\n\tcommands=(");
	script.push_str(&subcommand_names());
	let _ = writeln!(script, ")");
	let _ = writeln!(script, "\tif (( CURRENT == 2 )); then\n\t\tcompadd -- $commands {root}\n\t\treturn\n\tfi");
	let _ = writeln!(script, "\tcase \"${{words[2]}}\" in");
	let _ = writeln!(script, "\t\t(completions) compadd -- {} ;;", Shell::VALUES.join(" "));
	let _ = writeln!(script, "\t\t(langs) ;;");
	let _ = writeln!(script, "\t\t(diff) compadd -- {} ;;", longs_and_shorts(subcommand_spec("diff")));
	let _ = writeln!(script, "\t\t(init) compadd -- {} ;;", longs_and_shorts(subcommand_spec("init")));
	let _ = writeln!(script, "\tesac\n}}\ncompdef _cs cs\n");
	script
}

fn render_fish() -> String {
	let mut script = String::from("complete -c cs -f\n");
	for subcommand in ROOT_SPEC.subcommands {
		let _ = writeln!(script, "complete -c cs -f -n __fish_use_subcommand -a {} -d '{}'", subcommand.name, subcommand.about);
	}
	for option in ROOT_SPEC.options {
		let _ = write!(script, "complete -c cs -l {}", option.long);
		if let Some(short) = option.short {
			let _ = write!(script, " -s {short}");
		}
		if let Some(values) = option.possible_values {
			let _ = write!(script, " -x -a '{}'", values.join(" "));
		}
		script.push('\n');
	}
	for option in subcommand_spec("diff").options {
		let _ = write!(script, "complete -c cs -f -n '__fish_seen_subcommand_from diff' -l {}", option.long);
		if let Some(short) = option.short {
			let _ = write!(script, " -s {short}");
		}
		script.push('\n');
	}
	let _ = writeln!(script, "complete -c cs -f -n '__fish_seen_subcommand_from completions' -a '{}'", Shell::VALUES.join(" "));
	script
}

fn render_powershell() -> String {
	let mut words = vec![subcommand_names(), longs_and_shorts(&ROOT_SPEC)];
	words.push(Shell::VALUES.join(" "));
	words.push(longs_and_shorts(subcommand_spec("diff")));
	words.push(longs_and_shorts(subcommand_spec("init")));
	for values in ENUM_CHOICES {
		words.push(values.1.join(" "));
	}
	format!(
		"Register-ArgumentCompleter -Native -CommandName cs -ScriptBlock {{\n\tparam($wordToComplete, $commandAst, $cursorPosition)\n\t$candidates = @({})\n\t$candidates | Where-Object {{ $_.StartsWith($wordToComplete, [System.StringComparison]::OrdinalIgnoreCase) }}\n}}\n",
		words.into_iter().collect::<Vec<_>>().join(" "),
	)
}

fn render_elvish() -> String {
	let root = longs_and_shorts(&ROOT_SPEC);
	format!(
		"set edit:completion:arg-completer[cs] = {{|@args|\n\tif (== (count $args) 2) {{ put {} {} }}\n\telif (has-value $args completions) {{ put {} }}\n\telif (has-value $args diff) {{ put {} }}\n\telif (has-value $args init) {{ put {} }}\n\telse {{ put {} }}\n}}\n",
		subcommand_names(),
		root,
		Shell::VALUES.join(" "),
		longs_and_shorts(subcommand_spec("diff")),
		longs_and_shorts(subcommand_spec("init")),
		root,
	)
}
