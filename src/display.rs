//! Output formatting and display options for analysis results.
//!
//! ## Available Formatters
//!
//! - Human ([`HumanFormatter`]): Friendly, readable text output for terminal display.
//! - JSON ([`JsonFormatter`]): Pretty-printed JSON for easy processing and reading.
//! - JSON Compact ([`JsonCompactFormatter`]): Minified JSON for minimal bandwidth.
//!
//! ## Customization Options
//!
//! All formatters support common display options via [`ViewOptions`]:
//! - Number formatting: plain, comma-separated, underscore-separated, or space-separated.
//! - Size units: binary (KiB/MiB) or decimal (KB/MB).
//! - Percentage precision: 0-6 decimal places.
//! - Sort order: by lines, code, comments, blanks, files, size, or filename.
//! - Sort direction: ascending or descending.
//! - Indentation style: tab or 1-8 spaces.

mod diff;
pub mod formatting;
mod human;
mod json;
mod options;
mod report;

use std::{io::Write, path::PathBuf};

use anyhow::Result;
pub use diff::{Versions, write_diff};
pub use formatting::{FormatterContext, apply_sort};
pub use human::HumanFormatter;
pub use json::{JsonCompactFormatter, JsonFormatter};
pub use options::{IndentStyle, LanguageSortKey, NumberStyle, SizeStyle, SortDirection, Verbosity, ViewOptions};
pub use report::ReportData;
use serde::{Deserialize, Serialize};

use crate::analysis::AnalysisResults;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
	Human,
	Json,
	JsonCompact,
}

impl OutputFormat {
	pub const VALUES: &'static [&'static str] = &["human", "json", "json-compact"];

	#[must_use]
	pub fn from_cli(value: &str) -> Option<Self> {
		match value {
			"human" => Some(Self::Human),
			"json" => Some(Self::Json),
			"json-compact" => Some(Self::JsonCompact),
			_ => None,
		}
	}
}

/// Trait for formatting analysis results in a desired format.
pub trait OutputFormatter {
	/// Format and stream the analysis results to the provided writer
	///
	/// # Errors
	///
	/// Returns an error if formatting fails, (e.g. JSON serialization encounters an issue).
	fn write_output(
		&self,
		results: &AnalysisResults,
		paths: &[PathBuf],
		view_options: ViewOptions,
		writer: &mut dyn Write,
	) -> Result<()>;

	fn prepare_report<'a>(
		&self,
		results: &'a AnalysisResults,
		paths: &[PathBuf],
		view_options: ViewOptions,
	) -> (FormatterContext, ReportData<'a>) {
		let ctx = FormatterContext::new(view_options);
		let report = ReportData::from_results(results, paths, view_options.verbosity, &ctx);
		(ctx, report)
	}
}

/// Format and stream the analysis results to `writer` in the given format.
///
/// # Errors
///
/// Returns an error if formatting or writing fails.
pub fn write_output(
	format: OutputFormat,
	results: &AnalysisResults,
	paths: &[PathBuf],
	view_options: ViewOptions,
	writer: &mut dyn Write,
) -> Result<()> {
	match format {
		OutputFormat::Human => HumanFormatter.write_output(results, paths, view_options, writer),
		OutputFormat::Json => JsonFormatter.write_output(results, paths, view_options, writer),
		OutputFormat::JsonCompact => JsonCompactFormatter.write_output(results, paths, view_options, writer),
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn human_by_dir_replaces_the_language_breakdown() {
		use std::path::PathBuf;

		use super::{HumanFormatter, OutputFormatter, Verbosity, ViewOptions};
		use crate::analysis::{AnalysisResults, LineCounts};

		let mut results = AnalysisResults::default();
		let rust = crate::langs::LANGUAGES.iter().find(|l| l.name == "Rust").unwrap();
		let counts = LineCounts { total: 10, code: 10, ..LineCounts::default() };
		for path in ["src/main.rs", "tests/smoke.rs"] {
			results.add_file(rust, counts.into(), 100, Some(path.to_owned()));
		}
		let options = ViewOptions { by_dir: true, verbosity: Verbosity::Regular, ..ViewOptions::default() };
		let mut buf = Vec::new();
		HumanFormatter.write_output(&results, &[PathBuf::from(".")], options, &mut buf).unwrap();
		let output = String::from_utf8(buf).unwrap();
		assert!(output.contains("Directory breakdown:"), "{output}");
		assert!(output.contains("src") && output.contains("tests"), "{output}");
		assert!(!output.contains("Language breakdown:"), "{output}");
	}
}
