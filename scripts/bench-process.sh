#!/usr/bin/env bash
# Real-process benchmarks: startup and first-use costs that warm in-process benchmarks hide. Times the release binary on an empty directory, a single file, the fixture tree, and an 800-file generated Rust corpus.
# Usage: [CS=<binary>] scripts/bench-process.sh [scratch directory]

set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
case "${1:-$root/.work/bench-process}" in
	/*) work=${1:-$root/.work/bench-process};;
	*) work=$PWD/$1;;
esac
mkdir -p "$work"
cs=${CS:-}
if [[ -z $cs ]]; then
	cargo build --release --quiet --manifest-path "$root/Cargo.toml"
	cs=$root/target/release/cs
fi

mkdir -p "$work/empty"
printf 'fn main() {\n\t// one file\n\tlet answer = 42;\n\tprintln!("{answer}");\n}\n' >"$work/one.rs"
gen=$work/generated-rust
if [[ ! -d $gen ]]; then
	mkdir -p "$gen"
	for n in $(seq 0 799); do
		dir=$gen/src/mod$((n % 10))
		mkdir -p "$dir"
		{
			for i in $(seq 0 39); do
				printf 'fn function_%s(value: &str) -> usize {\n\tlet raw = r#"raw string with "quotes" and /* not a comment */"#;\n\t// comment %s\n\tlet s = "text %s";\n\tvalue.len() + s.len() + raw.len()\n}\n' "$i" "$i" "$i"
			done
		} >"$dir/file_$n.rs"
	done
fi

hyperfine -N --warmup 3 --runs 20 \
	-n empty "\"$cs\" \"$work/empty\"" \
	-n one-file "\"$cs\" \"$work/one.rs\"" \
	-n fixtures "\"$cs\" \"$root/tests/fixtures\"" \
	-n generated-800 "\"$cs\" \"$gen\""
