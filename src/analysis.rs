//! The analysis pipeline consists of several stages:
//!
//! 1. Directory Walking ([`analyzer`]): Parallel walking of the file tree ([`walk`]).
//! 2. I/O Strategy ([`file_io`]): Chooses between buffered and memory-mapped reading based on file size.
//! 3. Encoding Detection ([`encoding`]): Detects file encoding and filters out binary files.
//! 4. Line Classification ([`line_classifier`]): Categorizes each line as code, comment, blank, or shebang.
//! 5. Line Counting ([`line_counter`]): Accumulates line statistics for each file.
//! 6. Statistics Aggregation ([`stats`]): accumulation of code stats themselves, respecting the verbose setting.

mod analyzer;
mod attributes;
mod embedded;
mod encoding;
mod file_io;
pub mod generated;
mod git;
mod line_classifier;
mod line_counter;
mod list;
mod masks;
mod pipeline;
pub mod stats;
mod walk;

pub use analyzer::CodeAnalyzer;
pub use git::{Snapshot, count_snapshot};
pub use line_classifier::LineType;
pub use list::{count_listed, parse_list};
pub use stats::{AnalysisResults, Embedded, FileStats, LanguageStats, LineCounts};

/// Benchmarks and diagnostics reach the counting pipeline here. Nothing outside the crate should call this.
#[doc(hidden)]
pub mod bench {
	use super::{attributes, encoding, file_io, line_counter, pipeline, stats};

	/// Count `bytes` as `language`, detecting its encoding, as the pipeline would for a selected file.
	///
	/// # Panics
	///
	/// Panics where `language` isn't one of the known languages.
	#[must_use]
	pub fn count_as(language_name: &str, bytes: &[u8]) -> stats::FileCounts {
		let language = crate::langs::language_named(language_name).expect("known language");
		let sample_len = bytes.len().min(file_io::SAMPLE_SIZE * 2);
		let sample = &bytes[..sample_len];
		let encoding = encoding::detect_encoding(sample);
		line_counter::count_lines(language, encoding, bytes)
	}

	/// Detect `bytes`' language from `name` and count them, as the pipeline would for a file on disk.
	#[must_use]
	pub fn detect_and_count(
		name: &std::path::Path,
		bytes: &[u8],
	) -> Option<(&'static crate::langs::Language, stats::FileCounts)> {
		pipeline::count_bytes(
			name,
			bytes,
			pipeline::FileFilter {
				include: &[],
				exclude: &[],
				include_generated: true,
				attributes: None,
				extensions: &[],
				counted: None,
			},
			&mut attributes::DirCache::default(),
		)
	}
}
