use std::{
	collections::HashSet,
	path::Path,
};

use anyhow::{Result, ensure};

use crate::{
	config::Config,
	config_discovery,
	display::Verbosity,
	langs::language_named,
};

use super::{
	analyze::{AnalyzeArgs, SourceArgs, FormatArgs, ids},
	diff::DiffArgs,
};

macro_rules! apply {
	($seen:expr, $id:expr, $body:expr) => {
		if $seen.contains($id) {
			$body
		}
	};
}

/// Merge CLI arguments into `config`, with CLI taking precedence over file-based settings.
///
/// # Errors
///
/// Returns an error when the merged config sets both include and exclude languages.
pub fn merge_config(mut config: Config, args: &AnalyzeArgs, seen: &HashSet<&'static str>) -> Result<Config> {
	if seen.contains(ids::QUIET) && args.quiet {
		config.analysis.verbosity = Verbosity::Summary;
	}
	if seen.contains(ids::VERBOSE) && args.verbose {
		config.analysis.verbosity = Verbosity::Verbose;
	}
	apply!(seen, ids::NO_GITIGNORE, config.analysis.respect_gitignore = !args.no_gitignore);
	apply!(seen, ids::NO_GITATTRIBUTES, config.analysis.respect_gitattributes = !args.no_gitattributes);
	apply!(seen, ids::GIT, config.analysis.git_tracked_only = args.git);
	apply!(seen, ids::SYMLINKS, config.analysis.follow_symlinks = args.symlinks);
	apply!(seen, ids::PERCENT_PRECISION, config.display.precision = args.percent_precision);
	apply!(seen, ids::LANGUAGE_SORT, config.display.sort_by = args.language_sort);
	apply!(seen, ids::SORT_DIRECTION, config.display.sort_direction = args.sort_direction);
	apply!(seen, ids::MIN_LINES, config.display.min_lines = args.min_lines);
	apply!(seen, ids::BY_DIR, config.display.by_dir = args.by_dir);
	merge_shared(config, &args.source, &args.format, seen)
}

/// Merge `cs diff`'s arguments into `config`, as [`merge_config`] does for the main command.
///
/// # Errors
///
/// Returns an error when the merged config sets both include and exclude languages.
pub fn merge_diff_config(config: Config, args: &DiffArgs, seen: &HashSet<&'static str>) -> Result<Config> {
	merge_shared(config, &args.source, &args.format, seen)
}

fn merge_shared(mut config: Config, source: &SourceArgs, format: &FormatArgs, seen: &HashSet<&'static str>) -> Result<Config> {
	let paths_given = !source.paths.is_empty();
	if paths_given {
		config.paths.clone_from(&source.paths);
	}
	apply!(seen, ids::HIDDEN, config.analysis.include_hidden = source.hidden);
	apply!(seen, ids::INCLUDE_GENERATED, config.analysis.include_generated = source.include_generated);
	apply!(seen, ids::MAX_DEPTH, config.analysis.max_depth = source.max_depth);
	apply!(seen, ids::FAIL_ON_ERROR, config.analysis.fail_on_error = source.fail_on_error);
	apply!(seen, ids::NUMBER_STYLE, config.display.number_style = format.number_style);
	apply!(seen, ids::SIZE_STYLE, config.display.size_units = format.size_style);
	apply!(seen, ids::OUTPUT, config.display.output = format.output);
	apply!(seen, ids::INDENT, config.display.indent = format.indent);
	apply!(seen, ids::TOP_LANGUAGES, config.display.top_languages = format.top_languages);
	if seen.contains(ids::EXCLUDE) {
		config.analysis.exclude_patterns.extend(source.exclude.clone());
	}
	if seen.contains(ids::INCLUDE_LANG) {
		config.analysis.include_languages.extend(source.include_lang.clone());
	}
	if seen.contains(ids::EXCLUDE_LANG) {
		config.analysis.exclude_languages.extend(source.exclude_lang.clone());
	}
	if seen.contains(ids::EXT) {
		config.analysis.include_extensions.extend(source.ext.clone());
	}
	for extension in &mut config.analysis.include_extensions {
		*extension = extension.trim_start_matches('.').to_owned();
	}
	// Paths in a config file are relative to the file.
	if !paths_given
		&& config.paths_overridden
		&& let Some(parent) = config.source.as_deref().and_then(Path::parent)
	{
		for path in &mut config.paths {
			if path.is_relative() {
				*path = parent.join(&*path);
			}
		}
	}
	config.display.precision = config.display.precision.min(6);
	ensure!(
		config.display.top_languages != Some(0),
		"top_languages must be at least 1 (use verbosity = \"summary\" for totals only)"
	);
	ensure!(
		config.analysis.include_languages.is_empty() || config.analysis.exclude_languages.is_empty(),
		"Config cannot set both include_languages and exclude_languages"
	);
	for name in config.analysis.include_languages.iter().chain(&config.analysis.exclude_languages) {
		if language_named(name).is_none() {
			eprintln!("Warning: no language is named `{name}`. Run `cs langs` to list them.");
		}
	}
	Ok(config)
}

/// The config file given with `-c`, the defaults with `--no-config`, or else the file discovered.
pub fn base_config(source: &SourceArgs) -> Result<Config> {
	if source.no_config {
		return Ok(Config::default());
	}
	source.config.as_ref().map_or_else(config_discovery::load_default, Config::from_file)
}

pub fn ensure_path(config: &Config) -> Result<()> {
	for path in &config.paths {
		ensure!(path.exists(), "Path `{}` not found", path.display());
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use std::{
		fs,
		ffi::OsString,
		path::{Path, PathBuf},
	};

	use super::merge_config;
	use crate::{
		cli::{
			analyze::{AnalyzeArgs, parse_root},
			command::Flow,
		},
		config::Config,
		display::{IndentStyle, Verbosity},
	};

	fn parse_cli(arguments: &[&str]) -> (AnalyzeArgs, std::collections::HashSet<&'static str>) {
		let argv = arguments.iter().map(OsString::from).collect::<Vec<_>>();
		match parse_root("cs", false, argv).expect("arguments parse") {
			Flow::Done(parsed, seen) => (parsed, seen),
			_ => panic!("expected done"),
		}
	}

	fn write_config(contents: &str) -> PathBuf {
		static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
		let unique = std::sync::atomic::AtomicU64::fetch_add(&NEXT, 1, std::sync::atomic::Ordering::Relaxed);
		let root = Path::new(env!("CARGO_MANIFEST_DIR"))
			.join("target")
			.join("tmp")
			.join(format!("codestats_config_test_{}_{unique}", std::process::id()));
		fs::create_dir_all(&root).expect("create temp dir");
		let path = root.join("config.toml");
		fs::write(&path, contents).expect("write temp config");
		path
	}

	#[test]
	fn merge_resolves_config_relative_path() {
		let config_path = write_config("path = \"fixtures\"\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		let expected = config_path.parent().expect("config parent").join("fixtures");
		assert_eq!(merged.paths, [expected]);
	}

	#[test]
	fn config_takes_a_list_of_paths_relative_to_itself() {
		let config_path = write_config("paths = [\"src\", \"/abs\"]\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		let parent = config_path.parent().expect("config parent");
		// On Windows, a path that starts with a slash stays on the config's drive.
		let rooted = PathBuf::from("/abs");
		assert_eq!(merged.paths, [parent.join("src"), parent.join(rooted)]);
	}

	#[test]
	fn config_rejects_unknown_keys() {
		for contents in ["sort_by = \"code\"\n", "[display]\nsort_bye = \"code\"\n", "[analysis]\nhidden = true\n"] {
			let error = Config::from_file(write_config(contents)).expect_err(contents);
			assert!(format!("{error:#}").contains("unknown field"), "{contents}: {error:#}");
		}
	}

	#[test]
	fn config_rejects_an_empty_list_of_paths() {
		assert!(Config::from_file(write_config("paths = []\n")).is_err());
	}

	#[test]
	fn merge_preserves_cli_path_override() {
		let config_path = write_config("path = \"fixtures\"\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs", "custom-path"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.paths, [PathBuf::from("custom-path")]);
	}

	#[test]
	fn merge_clamps_precision() {
		let config_path = write_config("[display]\nprecision = 9\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.display.precision, 6);
	}

	#[test]
	fn merge_extends_exclude_patterns_from_cli() {
		let config_path = write_config("[analysis]\nexclude_patterns = [\"target\"]\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs", "--exclude", "node_modules"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.analysis.exclude_patterns, vec!["target".to_string(), "node_modules".to_string()]);
	}

	#[test]
	fn merge_rejects_conflicting_language_filters() {
		let config_path = write_config("[analysis]\ninclude_languages = [\"Rust\"]\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs", "--exclude-lang", "python"]);
		let err = merge_config(config, &args, &seen).expect_err("conflicting language filters");
		assert_eq!(err.to_string(), "Config cannot set both include_languages and exclude_languages");
	}

	#[test]
	fn merge_applies_boolean_overrides() {
		let config_path = write_config("[analysis]\nrespect_gitignore = true\ninclude_hidden = false\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs", "--no-gitignore", "--hidden"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert!(!merged.analysis.respect_gitignore);
		assert!(merged.analysis.include_hidden);
	}

	#[test]
	fn merge_applies_indent_from_cli() {
		let config_path = write_config("");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs", "--indent", "4"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.display.indent, IndentStyle::Spaces(4));
	}

	#[test]
	fn merge_applies_indent_from_config() {
		let config_path = write_config("[display]\nindent = \"2\"\n");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.display.indent, IndentStyle::Spaces(2));
	}

	#[test]
	fn merge_indent_defaults_to_tab() {
		let config_path = write_config("");
		let config = Config::from_file(&config_path).expect("load config");
		let (args, seen) = parse_cli(&["cs"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.display.indent, IndentStyle::Tab);
	}

	#[test]
	fn merge_applies_verbosity_overrides() {
		let config_path = write_config("[analysis]\nverbosity = \"regular\"\n");
		let config = Config::from_file(&config_path).expect("load config");

		let (args, seen) = parse_cli(&["cs", "--quiet"]);
		let merged = merge_config(config.clone(), &args, &seen).expect("merge config");
		assert_eq!(merged.analysis.verbosity, Verbosity::Summary);

		let (args, seen) = parse_cli(&["cs", "--verbose"]);
		let merged = merge_config(config, &args, &seen).expect("merge config");
		assert_eq!(merged.analysis.verbosity, Verbosity::Verbose);
	}
}
