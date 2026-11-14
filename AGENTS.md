# loc

`loc` counts code, comment, blank, and shebang lines in a tree, by language, and reports them as text or JSON. User-facing usage is the [README](README.md); this file covers working on the crate itself.

## Commands

Use the toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml).

```sh
make build            # cargo build
make test             # the test suite
make check            # clippy with -D warnings, plus tests
make lint             # check, plus ruff on scripts
make bench            # compare the benchmark suite against benches/baseline.json
make release TARGET=x86_64-unknown-linux-musl     # the static dist binary (Dockerfile)
make verify-release TARGET=x86_64-unknown-linux-musl
```

Linux release binaries are built with `-Z build-std` in a pinned Alpine container ([Dockerfile](Dockerfile)); `make verify-release` checks their ELF linking. `make install` installs the dist binary to `~/.local/bin`.

Prereleases are cut by dispatching the `release` workflow on the default branch; each dispatch publishes an archive per platform with a `SHA256SUMS` file.

The [Makefile](Makefile) is the entry point; per-task commands are `cargo build --release`, `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo bench --bench bench` for the rustybench harness.

Keep scratch under the ignored `.work/`: `TMPDIR="$PWD/.work/tmp" make test`.

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
