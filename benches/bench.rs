//! Benchmark harness for codestats.
//!
//! Tracks wall time and allocations on the paths that matter for a counter of
//! real trees: walking and classifying a corpus, and rendering the reports.
//! Run: `cargo bench --bench bench`.

use std::{
	fmt::Write as _,
	fs,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use rustybench::{AllocProfiler, Bencher, black_box};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

use codestats::{
	analysis::CodeAnalyzer,
	config::Config,
	display::{OutputFormat, ViewOptions, write_output},
};

struct BenchDir(PathBuf);

impl BenchDir {
	fn new(label: &str) -> Self {
		let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let path = std::env::temp_dir().join(format!("codestats-bench-{label}-{}-{stamp}", std::process::id()));
		fs::create_dir_all(&path).unwrap();
		Self(path)
	}

	fn path(&self) -> &Path {
		&self.0
	}
}

impl Drop for BenchDir {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.0);
	}
}

/// A tree of generated Rust files with comments, strings, and raw strings, the shape of a real source tree.
fn generated_corpus(label: &str) -> BenchDir {
	let corpus = BenchDir::new(label);
	let mut body = String::new();
	for i in 0..40 {
		writeln!(body, "fn function_{i}(value: &str) -> usize {{").unwrap();
		writeln!(body, "\tlet raw = r#\"raw string with \"quotes\" and /* not a comment */\"#;").unwrap();
		writeln!(body, "\t// comment {i}").unwrap();
		writeln!(body, "\tlet s = \"text {i}\";").unwrap();
		writeln!(body, "\tvalue.len() + s.len() + raw.len()").unwrap();
		writeln!(body, "}}").unwrap();
	}
	for n in 0..800 {
		let dir = corpus.path().join(format!("src/mod{}", n % 10));
		fs::create_dir_all(&dir).unwrap();
		fs::write(dir.join(format!("file_{n}.rs")), &body).unwrap();
	}
	corpus
}

fn fixtures() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn count(root: &Path) -> codestats::analysis::AnalysisResults {
	let config = Config::default();
	CodeAnalyzer::new(root, (&config).into()).analyze().unwrap()
}

#[rustybench::bench]
fn count_generated_corpus(bencher: Bencher) {
	let corpus = generated_corpus("corpus");
	bencher.bench_local(|| black_box(count(corpus.path())));
}

#[rustybench::bench]
fn count_fixtures(bencher: Bencher) {
	let root = fixtures();
	bencher.bench_local(|| black_box(count(&root)));
}

#[rustybench::bench]
fn render_json_compact(bencher: Bencher) {
	let root = fixtures();
	let results = count(&root);
	bencher.bench_local(|| {
		let mut buf = Vec::new();
		write_output(OutputFormat::JsonCompact, &results, std::slice::from_ref(&root), ViewOptions::default(), &mut buf).unwrap();
		black_box(buf);
	});
}

#[rustybench::bench]
fn render_human(bencher: Bencher) {
	let root = fixtures();
	let results = count(&root);
	bencher.bench_local(|| {
		let mut buf = Vec::new();
		write_output(OutputFormat::Human, &results, std::slice::from_ref(&root), ViewOptions::default(), &mut buf).unwrap();
		black_box(buf);
	});
}

fn main() {
	rustybench::main();
}
