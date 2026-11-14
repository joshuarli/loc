//! Benchmark harness for loc.
//!
//! Tracks wall time and allocations on the paths that matter for a counter of
//! real trees: walking and classifying a corpus, and rendering the reports.
//! Run: `cargo bench --bench bench`.

use std::{
	fmt::Write as _,
	fs,
	io::Write as _,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use rustybench::{AllocProfiler, Bencher, black_box};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

use laputa_loc::{
	analysis::CodeAnalyzer,
	config::Config,
	display::{OutputFormat, ViewOptions, write_output},
};

struct BenchDir(PathBuf);

impl BenchDir {
	fn new(label: &str) -> Self {
		let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(".work").join("bench");
		fs::create_dir_all(&root).unwrap();
		let path = root.join(format!("loc-bench-{label}-{}-{stamp}", std::process::id()));
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
	let body = rust_body();
	for n in 0..800 {
		let dir = corpus.path().join(format!("src/mod{}", n % 10));
		fs::create_dir_all(&dir).unwrap();
		fs::write(dir.join(format!("file_{n}.rs")), &body).unwrap();
	}
	corpus
}

/// A Rust source body with comments, strings and raw strings, the shape of a real source file.
fn rust_body() -> String {
	let mut body = String::new();
	for i in 0..40 {
		writeln!(body, "fn function_{i}(value: &str) -> usize {{").unwrap();
		writeln!(body, "\tlet raw = r#\"raw string with \"quotes\" and /* not a comment */\"#;").unwrap();
		writeln!(body, "\t// comment {i}").unwrap();
		writeln!(body, "\tlet s = \"text {i}\";").unwrap();
		writeln!(body, "\tvalue.len() + s.len() + raw.len()").unwrap();
		writeln!(body, "}}").unwrap();
	}
	body
}

/// About 256 KiB of Rust: the body repeated, still valid source, all one module's worth of items.
fn large_rust() -> Vec<u8> {
	let body = rust_body();
	let repeat = (256usize * 1024).div_ceil(body.len());
	body.repeat(repeat).into_bytes()
}

fn fixtures() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn count(root: &Path) -> laputa_loc::analysis::AnalysisResults {
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
fn count_rust_in_memory(bencher: Bencher) {
	let bytes = large_rust();
	// Warm the lazy per-language matchers before the measured loop, so first-use is not charged to steady-state scanning.
	black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes)));
}

#[rustybench::bench]
fn detect_and_count_rust_in_memory(bencher: Bencher) {
	let bytes = large_rust();
	let name = Path::new("bench.rs");
	black_box(laputa_loc::analysis::bench::detect_and_count(name, &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::detect_and_count(name, &bytes)));
}

#[rustybench::bench]
fn count_blank_heavy_in_memory(bencher: Bencher) {
	// Mostly-newline input: exercises the blank-line accounting in the mask sweep.
	let bytes = b"\n\n\n\nfn f() {}\n\n\n\nlet x = 1;\n\n\n\n".repeat(8 * 1024);
	black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes)));
}

#[rustybench::bench]
fn count_minified_in_memory(bencher: Bencher) {
	// One long line with dense punctuation: no newlines, markers every few bytes.
	let mut bytes = Vec::with_capacity(128 * 1024);
	while bytes.len() < 128 * 1024 {
		bytes.extend_from_slice(b"var x={a:1,b:\"s\",c:[1,2,3]};function f(){return x.a+`t${x.b}u`}");
	}
	black_box(laputa_loc::analysis::bench::count_as("JavaScript", &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::count_as("JavaScript", &bytes)));
}

#[rustybench::bench]
fn count_non_ascii_in_memory(bencher: Bencher) {
	// Multi-byte UTF-8 throughout, including inside comments and strings.
	let line = "// αβγδ εζηθ -> 'λ' + \"μνξο\" = π\nlet σύμβολο = \"αβγδ\";\n\n";
	let bytes = line.repeat(8 * 1024).into_bytes();
	black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes)));
}

#[rustybench::bench]
fn count_r_dense_in_memory(bencher: Bencher) {
	// One long line of r-heavy words ending in a marker: every candidate search on it finds "r" hundreds of times before the comment delimiter.
	let mut bytes = String::with_capacity(48 * 1024);
	for _ in 0..700 {
		bytes.push_str("r rr rrr rrrr rrrrr ");
	}
	bytes.push_str("// tail comment\n");
	let bytes = bytes.into_bytes();
	black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::count_as("Rust", &bytes)));
}

#[rustybench::bench]
fn count_json_in_memory(bencher: Bencher) {
	let mut bytes = Vec::with_capacity(256 * 1024);
	let mut n = 0u64;
	while bytes.len() < 256 * 1024 {
		n += 1;
		writeln!(bytes, "{{\"key{n}\": [{n}, \"value{n}\", true, null], \"other\": {{\"nested\": \"αβ\"}}}},").unwrap();
	}
	black_box(laputa_loc::analysis::bench::count_as("JSON", &bytes));
	bencher.bench_local(|| black_box(laputa_loc::analysis::bench::count_as("JSON", &bytes)));
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
