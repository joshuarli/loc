# loc

`loc` counts code, comment, blank, and shebang lines in a tree, by language, and reports them as text or JSON. User-facing usage is the [README](README.md); this file covers working on the crate itself.

The roadmap, [docs/roadmap.md](docs/roadmap.md), holds all current and future work; its Status is the resume point.

## Commands

Use the toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml).

```sh
make build            # cargo build
make clean            # remove this checkout's ./target build directory
make test             # the test suite
make check            # clippy with -D warnings, plus tests
make lint             # check, plus ruff on scripts
make bench            # benchmark driver self-tests; see docs/benchmarks.md for tiers
make release TARGET=x86_64-unknown-linux-musl     # the static dist binary (Dockerfile)
make verify-release TARGET=x86_64-unknown-linux-musl
```

Linux release binaries are built with `-Z build-std` in a pinned Alpine container ([Dockerfile](Dockerfile)); `make verify-release` checks their ELF linking. `make install` installs the dist binary to `~/.local/bin`.

Prereleases are cut by dispatching the `release` workflow on the default branch; each dispatch publishes an archive per platform with a `SHA256SUMS` file.

The [Makefile](Makefile) is the entry point; per-task commands are `cargo build --release`, `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo bench --bench bench` for the rustybench harness.

Keep scratch under the ignored `.work/`: `TMPDIR="$PWD/.work/tmp" make test`.
`make clean` removes the checkout's ordinary `./target` directory and refuses files or
symlinks at that path. Verify it in a disposable checkout with a generated file in `target/`,
a nested `target/` fixture, and an external `CARGO_TARGET_DIR`. Repeat the command and verify
the nested fixture and external target survive. A root symlink to the external target must fail.

## Verification

- Edit: `cargo test --test cli top_level_help_is_complete_and_has_owned_footer -- --exact` is a focused CLI help check. Use `make test` for Change.
- At base `9ea3b7a`, `make test` took 10.16s with a fresh target and 1.56s warm; `make lint` took under 4s (Clippy dominates). Set `TMPDIR="$PWD/.work/tmp"`.
- CLI analysis, paths, filters, stdin and diff: `tests/cli.rs` and `tests/fixture_counts.rs`; flag parsing and help: `src/cli/command.rs` and `src/cli/mod.rs` tests. Individual flag combinations are not all covered.
- CLI help and version: `top_level_help_is_complete_and_has_owned_footer`, `every_command_has_offline_help_before_required_arguments`, `version_reports_the_package_version_and_a_commit`; language listing: `src/langs.rs` tests; init: `src/init.rs` tests.
- Exit codes 0, 1 and 2: `tests/cli.rs`. `unreadable_file_reports_output_and_fail_on_error_status` proves the protected-file exit and diagnostic.
- Human and JSON output, including diff: `tests/cli.rs` and `src/display/json.rs`. `complete_multilanguage_reports` checks complete analysis reports.
- Local, platform and home configuration precedence: `configuration_discovery_prefers_local_then_platform_then_home`. TOML merge and init: `src/cli/merge.rs` and `src/init.rs`. JSON schema: `schema_has_one_shape_for_counts_at_every_level`.
- Public Rust modules and `run`: in-process unit tests exercise parts of the API; external consumer contract test: none. Known consumers: this crate's binary and `benches/bench.rs`; other consumers are unknown.
- Network endpoints: none. Counting semantics: `tests/fixture_counts.rs` and the CI LineJudge check.
- Scanner tests return early when required CPU features are absent; verify x86 scanner behavior on a host with AVX2, POPCNT, LZCNT and BMI1. Run in-place mutation tests separately from ordinary builds, since concurrent builds can compile mutated source.
- Performance workload: `count_generated_corpus` in `benches/bench.rs` walks and counts 800 generated Rust files; `make bench TIER=full` compares `benches/baseline.json`; `make bench TIER=change WORKLOAD=count_generated_corpus` runs one short workload.

## Invariants

- Windows is not supported; the crate fails to compile on it.
- JSON fields and configuration keys only change in a major release; `schema_version` bumps whenever a field is renamed, removed, or changes meaning.
- Exit codes: `0` success, `1` an error (including unreadable files with `--fail-on-error`), `2` invalid arguments.
- `--version` prints `loc <version> (git <sha>)` with the commit embedded at build time, from git, from `.cargo_vcs_info.json` in a crate archive, or from `LOC_BUILD_SHA`; otherwise the build fails rather than embed a wrong commit.
- Unit tests write scratch under `target/tmp`; integration tests under `CARGO_TARGET_TMPDIR`; nothing else writes there.
- Paths given on the command line are counted once however they are spelled; `-` reads the list from standard input, one per line or NUL-separated.
- Tests run against the built binary (`tests/cli.rs`) and in-process unit tests; the counting rules are additionally pinned by LineJudge (see the README).

## Building

```sh
git clone https://github.com/joshuarli/loc
cd loc
cargo install --path .
```

`loc --version` reports the commit the binary was built from, embedded at build time. Building outside a git checkout, for example from a source archive, requires the commit id to be set explicitly:

```sh
LOC_BUILD_SHA=<7 to 40 digit commit id> cargo install --path .
```

If neither a git checkout nor a Cargo archive is available and `LOC_BUILD_SHA` isn't set, the build fails with an error rather than embedding a wrong commit.

## Map

- `src/lib.rs` — the crate root; `pub fn run` ties the binary to the library
- `src/cli/` — the command surface: schema-driven parsing, help, errors, and exit paths (`lexopt` is the only parser)
- `src/analysis/` — walking, detection, and line counting
- `src/display/` — human and JSON reports, and the diff report
- `build.rs` — compiles `languages.json5` into the language table and embeds the version commit
- `benches/` — rustybench harness and the recorded baseline
- `.linejudge/` — conformance suite for the counting rules; regenerate `.linejudge/recorded/` only deliberately
