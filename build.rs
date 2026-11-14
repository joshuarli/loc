//! Compiles `languages.json5` into a static Rust table of language definitions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[path = "build/config.rs"]
mod config;
#[path = "build/render.rs"]
mod render;
#[path = "build/validate.rs"]
mod validate;

use std::{
	env,
	error::Error,
	fs,
	path::{Path, PathBuf},
	process::Command,
	result,
};

use indexmap::IndexMap;

use crate::{
	config::LanguageConfig,
	render::render_languages,
	validate::{normalize_languages, validate_languages},
};

type Result<T> = result::Result<T, Box<dyn Error>>;

const BUILD_SHA_VAR: &str = "LOC_BUILD_SHA";

fn main() -> Result<()> {
	emit_git_sha();
	let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
	let json_path = Path::new(&manifest_dir).join("languages.json5");
	println!("cargo:rerun-if-changed={}", json_path.display());
	// Declaring any rerun-if-changed replaces cargo's default of watching the whole package, so the build script's own modules have to be listed too.
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=build");
	let json_content = fs::read_to_string(&json_path)?;
	let language_map: IndexMap<String, LanguageConfig> =
		json5::from_str(&json_content).map_err(|e| format!("Failed to parse languages.json5: {e}"))?;
	let languages = normalize_languages(language_map)?;
	validate_languages(&languages)?;
	let out_dir = env::var("OUT_DIR")?;
	fs::write(Path::new(&out_dir).join("languages.rs"), render_languages(&languages))?;
	Ok(())
}

fn emit_git_sha() {
	let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
	let checkout = Path::new(&manifest_dir);
	let sha = env::var(BUILD_SHA_VAR).ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
		git_output(checkout, &["rev-parse", "--short=12", "HEAD"])
			.or_else(|| vcs_info_sha(checkout))
			.unwrap_or_else(|| panic!("couldn't determine the git commit to embed in --version; build in a git checkout, or set {BUILD_SHA_VAR} to a commit id (required for release archives)"))
	});
	let sha = validate_sha(&sha);
	println!("cargo:rustc-env=LOC_GIT_SHA={sha}");
	println!("cargo:rerun-if-env-changed={BUILD_SHA_VAR}");
	if let Some((git_dir, common_dir, reference)) = git_dirs(checkout) {
		println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
		if let Some(reference) = reference {
			println!("cargo:rerun-if-changed={}", common_dir.join(reference).display());
		}
		let packed_refs = common_dir.join("packed-refs");
		if packed_refs.exists() {
			println!("cargo:rerun-if-changed={}", packed_refs.display());
		}
	}
}

#[derive(serde::Deserialize)]
struct VcsInfo {
	git: Option<VcsGit>,
}

#[derive(serde::Deserialize)]
struct VcsGit {
	sha1: String,
	dirty: Option<bool>,
}

fn vcs_info_sha(checkout: &Path) -> Option<String> {
	let text = fs::read_to_string(checkout.join(".cargo_vcs_info.json")).ok()?;
	let info: VcsInfo = json5::from_str(&text).ok()?;
	let git = info.git?;
	if git.dirty == Some(true) {
		return None;
	}
	let sha = git.sha1.trim().to_owned();
	if (7..=40).contains(&sha.len()) && sha.bytes().all(|b| b.is_ascii_hexdigit()) {
		Some(sha[..12].to_owned())
	} else {
		None
	}
}

fn git_output(dir: &Path, args: &[&str]) -> Option<String> {
	let output = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
	if !output.status.success() {
		return None;
	}
	Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn git_dirs(checkout: &Path) -> Option<(PathBuf, PathBuf, Option<String>)> {
	let raw_git_dir = git_output(checkout, &["rev-parse", "--git-dir"])?;
	let git_dir = absolutize(checkout, PathBuf::from(raw_git_dir));
	let raw_common = git_output(checkout, &["rev-parse", "--git-common-dir"])?;
	let common_dir = absolutize(checkout, PathBuf::from(raw_common));
	let reference = fs::read_to_string(git_dir.join("HEAD"))
		.ok()
		.and_then(|head| head.strip_prefix("ref: ").map(|r| r.trim().to_owned()));
	Some((git_dir, common_dir, reference))
}

fn absolutize(checkout: &Path, path: PathBuf) -> PathBuf {
	if path.is_absolute() { path } else { checkout.join(path) }
}

fn validate_sha(sha: &str) -> String {
	let sha = sha.trim();
	assert!(
		(7..=40).contains(&sha.len()) && sha.bytes().all(|b| b.is_ascii_hexdigit()),
		"the commit id {sha:?} is not a 7 to 40 digit hexadecimal id; refusing to embed it in --version output"
	);
	sha.to_owned()
}
