pub const GIT_SHA: &str = env!("LOC_GIT_SHA");

pub fn version_line() -> String {
	fmt_version(env!("CARGO_PKG_VERSION"), GIT_SHA)
}

fn fmt_version(version: &str, sha: &str) -> String {
	format!("loc {version} (git {sha})")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn version_line_carries_the_package_version() {
		assert_eq!(version_line(), fmt_version(env!("CARGO_PKG_VERSION"), GIT_SHA));
	}

	#[test]
	fn git_sha_is_hexadecimal() {
		assert!(
			(7..=40).contains(&GIT_SHA.len()) && GIT_SHA.bytes().all(|b| b.is_ascii_hexdigit()),
			"embedded sha {GIT_SHA:?} is not a hexadecimal commit id"
		);
	}
}
