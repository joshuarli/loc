//! Output for `loc diff`: each language's counts in two versions, side by side, as text or JSON.

use std::{cmp::Reverse, fmt::Write as _, io::Write, path::Path};

use anyhow::Result;
use serde::Serialize;

use super::{FormatterContext, OutputFormat, ViewOptions, formatting::pluralize};
use crate::{
	analysis::{AnalysisResults, LineCounts, LineType},
	langs::LANGUAGES,
};

/// Version of the `loc diff` JSON shape, raised whenever a field is renamed, removed or changes meaning.
const SCHEMA_VERSION: u32 = 2;

/// One version's counts for a language, or for everything.
#[derive(Clone, Copy, Default, Serialize)]
struct Side {
	file_count: u64,
	lines: LineCounts,
	bytes: u64,
}

struct Row {
	name: &'static str,
	/// `None` when the language isn't in that version.
	from: Option<Side>,
	to: Option<Side>,
}

impl Row {
	fn line_change(&self) -> i128 {
		i128::from(self.to.unwrap_or_default().lines.total) - i128::from(self.from.unwrap_or_default().lines.total)
	}

	const fn status(&self) -> &'static str {
		match (self.from, self.to) {
			(None, Some(_)) => " (new)",
			(Some(_), None) => " (removed)",
			_ => "",
		}
	}
}

/// Two versions being compared: their names (`None` for the working tree) and counts.
pub struct Versions<'a> {
	pub from_name: &'a str,
	pub to_name: Option<&'a str>,
	pub from: &'a AnalysisResults,
	pub to: &'a AnalysisResults,
}

struct Diff<'a> {
	path: String,
	from_name: &'a str,
	to_name: Option<&'a str>,
	totals: (Side, Side),
	rows: Vec<Row>,
	hidden: usize,
}

impl<'a> Diff<'a> {
	fn new(versions: &Versions<'a>, path: &Path, options: ViewOptions) -> Self {
		let side = |results: &AnalysisResults, index| {
			results.languages().find(|(lang, _)| lang.index == index).map(|(_, stats)| Side {
				file_count: stats.files(),
				lines: *stats.counts(),
				bytes: stats.size(),
			})
		};
		let mut rows: Vec<Row> = LANGUAGES
			.iter()
			.map(|lang| Row {
				name: lang.name,
				from: side(versions.from, lang.index),
				to: side(versions.to, lang.index),
			})
			.filter(|row| row.from.is_some() || row.to.is_some())
			.collect();
		// Biggest changes first; languages that didn't change at all come last, largest first.
		rows.sort_by_key(|row| {
			(
				Reverse(row.line_change().unsigned_abs()),
				Reverse(row.to.or(row.from).unwrap_or_default().lines.total),
				row.name,
			)
		});
		let total = rows.len();
		if let Some(n) = options.top_languages {
			rows.truncate(n);
		}
		let totals = |results: &AnalysisResults| Side {
			file_count: results.total_files(),
			lines: *results.counts(),
			bytes: results.total_size(),
		};
		Self {
			path: path.display().to_string(),
			from_name: versions.from_name,
			to_name: versions.to_name,
			totals: (totals(versions.from), totals(versions.to)),
			hidden: total - rows.len(),
			rows,
		}
	}

	fn to_description(&self) -> String {
		self.to_name.map_or_else(|| "the working tree".to_owned(), |name| format!("`{name}`"))
	}
}

/// A change between two counts, signed: `+21`, `-3`, or `0`.
fn change(ctx: &FormatterContext, from: u64, to: u64) -> String {
	match to.cmp(&from) {
		std::cmp::Ordering::Greater => format!("+{}", ctx.number(to - from)),
		std::cmp::Ordering::Less => format!("-{}", ctx.number(from - to)),
		std::cmp::Ordering::Equal => "0".to_owned(),
	}
}

/// "from → to (change)" for a table cell.
fn cell(ctx: &FormatterContext, from: u64, to: u64) -> String {
	if from == to {
		ctx.number(to)
	} else {
		format!("{} → {} ({})", ctx.number(from), ctx.number(to), change(ctx, from, to))
	}
}

fn size_cell(ctx: &FormatterContext, from: u64, to: u64) -> String {
	if from == to {
		return ctx.size(to);
	}
	let sign = if to > from { '+' } else { '-' };
	format!("{} → {} ({sign}{})", ctx.size(from), ctx.size(to), ctx.size(from.abs_diff(to)))
}

const LINE_TYPES: [LineType; 4] = [LineType::Code, LineType::Comment, LineType::Blank, LineType::Shebang];

/// The line types that changed, like "+900 code, +80 comments".
fn line_type_changes(ctx: &FormatterContext, from: &LineCounts, to: &LineCounts) -> Vec<String> {
	LINE_TYPES
		.iter()
		.filter(|&&kind| from.get(kind) != to.get(kind))
		.map(|&kind| {
			let (from, to) = (from.get(kind), to.get(kind));
			format!(
				"{} {}",
				change(ctx, from, to),
				pluralize(from.abs_diff(to), kind.singular_label(), kind.plural_label())
			)
		})
		.collect()
}

/// Write a comparison of two versions' counts.
///
/// # Errors
///
/// Returns an error if writing fails.
pub fn write_diff(
	format: OutputFormat,
	versions: &Versions,
	path: &Path,
	options: ViewOptions,
	writer: &mut dyn Write,
) -> Result<()> {
	let ctx = FormatterContext::new(options);
	let diff = Diff::new(versions, path, options);
	match format {
		OutputFormat::Human => write_human(&diff, &ctx, writer),
		OutputFormat::Json => write_json(&diff, options, writer, true),
		OutputFormat::JsonCompact => write_json(&diff, options, writer, false),
	}
}

fn write_human(diff: &Diff, ctx: &FormatterContext, writer: &mut dyn Write) -> Result<()> {
	let (from, to) = diff.totals;
	writeln!(writer, "Loc diff for {} from `{}` to {}:", diff.path, diff.from_name, diff.to_description())?;
	let mut lines = format!(
		"Files: {}. Lines: {}",
		cell(ctx, from.file_count, to.file_count),
		cell(ctx, from.lines.total, to.lines.total)
	);
	let types = line_type_changes(ctx, &from.lines, &to.lines);
	if !types.is_empty() {
		write!(lines, ": {}", types.join(", "))?;
	}
	writeln!(writer, "{lines}. Size: {}.", size_cell(ctx, from.bytes, to.bytes))?;
	if diff.rows.is_empty() {
		writeln!(writer, "No recognized programming languages found.")?;
		return Ok(());
	}
	writeln!(writer, "Languages:")?;
	let indent = ctx.indent(1);
	for row in &diff.rows {
		let (from, to) = (row.from.unwrap_or_default(), row.to.unwrap_or_default());
		let mut line = format!(
			"{indent}{}{}: {} {}, {} {}",
			row.name,
			row.status(),
			cell(ctx, from.lines.total, to.lines.total),
			pluralize(to.lines.total.max(from.lines.total), "line", "lines"),
			cell(ctx, from.file_count, to.file_count),
			pluralize(to.file_count.max(from.file_count), "file", "files"),
		);
		let types = line_type_changes(ctx, &from.lines, &to.lines);
		if !types.is_empty() {
			write!(line, "; {}", types.join(", "))?;
		}
		writeln!(writer, "{line}.")?;
	}
	if diff.hidden > 0 {
		writeln!(
			writer,
			"({} more {} not shown)",
			diff.hidden,
			pluralize(diff.hidden as u64, "language", "languages")
		)?;
	}
	Ok(())
}

#[derive(Serialize)]
struct JsonDiff<'a> {
	schema_version: u32,
	/// The paths compared, as a list like the main report's, though `loc diff` takes one.
	paths: [&'a str; 1],
	from: &'a str,
	/// `None` for the working tree.
	to: Option<&'a str>,
	totals: JsonPair,
	languages: Vec<JsonLanguage>,
	hidden_languages: usize,
}

#[derive(Serialize)]
struct JsonPair {
	from: Side,
	to: Side,
}

#[derive(Serialize)]
struct JsonLanguage {
	name: &'static str,
	/// `None` when the language isn't in that version.
	from: Option<Side>,
	to: Option<Side>,
}

fn write_json(diff: &Diff, options: ViewOptions, writer: &mut dyn Write, pretty: bool) -> Result<()> {
	let json = JsonDiff {
		schema_version: SCHEMA_VERSION,
		paths: [&diff.path],
		from: diff.from_name,
		to: diff.to_name,
		totals: JsonPair { from: diff.totals.0, to: diff.totals.1 },
		languages: diff.rows.iter().map(|row| JsonLanguage { name: row.name, from: row.from, to: row.to }).collect(),
		hidden_languages: diff.hidden,
	};
	super::json::write_value(&json, options, writer, pretty)
}


