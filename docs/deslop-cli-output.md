# CLI output and unreadable-file contract

DSLP-4 identifies missing whole human/JSON reports and an unreadable-file exit witness (TODO-DESLOP.md, Tests). Existing CLI tests check individual fields and fragments; the JSON formatter unit tests check schema shape. The two new binary tests use literal Rust and Python files, assert the entire verbose human report and entire JSON values in both encodings, and check status and stderr. The permission test checks that mode 000 actually denies opening, then covers skipped and clean input with the flag and skipped input without it. It restores permissions before assertions on the subprocess results. It requires a native Unix user for whom mode 000 denies access; running as a privileged user fails the fixture check rather than silently passing. No HOME override is used. The output JSON comparison is structural, not a whitespace or key-order contract.

Base `f5603d85b27d53464c1c9836e0a72b171efc25f3`; production `src/cli/analyze.rs` SHA-256 before and after `844a5aa8a861aa55bdbf921f491d0b60c23e14a1b7a107b652df415091b97657`. No production behavior changed. Darwin arm64, rustc 1.102.0-nightly (5b7b37aef 2026-11-08), cargo-mutants 27.1.0, default features. This is a bounded CLI slice, not closure of DSLP-4, Linux qualification, or a CI mutation lane.

## Bounded mutation inventory

The component is `execute` in `src/cli/analyze.rs`, including selection of listed input vs roots, multiple-root handling, and the post-output skipped-entry decision. Output formatting itself has no selected operator mutant here; the hand faults below exercise its dispatch. The tool selected all nine identities despite the `--re` argument. Each identity and before → after disposition:

| Identity (line:column in unchanged production) | Before → after |
| --- | --- |
| 303:2 replace body with `Ok(())` | unviable → unviable: unused imports, arguments and dead functions under denied warnings |
| 304:67 `==` to `!=` | caught → caught |
| 319:34 `>` to `==` | caught → caught |
| 319:34 `>` to `<` | caught → caught |
| 319:34 `>` to `>=` | caught → caught |
| 328:35 `&&` to `||` | missed → caught: flag absent with skipped file must exit 0 |
| 328:64 `>` to `==` | missed → caught: flag present with skipped file must exit 1 |
| 328:64 `>` to `<` | unviable → unviable: useless unsigned comparison denied |
| 328:64 `>` to `>=` | unviable → unviable: useless unsigned comparison denied |

Baseline exit 2, 4 caught, 2 missed, 3 unviable; final exit 0, 6 caught, 0 missed, 3 unviable; no timeouts. Baselines passed. Runs took 41s and 27s; an intermediate candidate run took 28s, total 96s, below the 20-minute mutation budget. Logs for the two newly caught mutants show the new status assertions failing with 1 instead of 0 and 0 instead of 1, respectively. No survivor is treated as detected by an unviable build.

Reproduce each run with distinct `OUT` (before requires the base tests; after requires the candidate tests). No ordinary build may overlap this in-place run. The lock acquisition must succeed and only its owner releases it:

```sh
mkdir -p .work/tmp
mkdir /Users/josh/d/stuff/.work/tmp/mutation.lock || exit 1
trap 'rmdir /Users/josh/d/stuff/.work/tmp/mutation.lock' EXIT
export TMPDIR="$PWD/.work/tmp" CARGO_TARGET_DIR="$PWD/.work/cli-mut-target"
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 RUSTC_WRAPPER=kache
OUT=.work/cli-result
timeout -k 5 1200 cpu-guard --cores 2 -- /Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --in-place --no-config --skip-calls-defaults=false --file src/cli/analyze.rs -F 'execute' --re '328:' --baseline run --timeout 60 --build-timeout 300 --no-shuffle --output "$OUT" -- --test cli
```

## Fault controls and checks

With the same lock and mutation target, replace the `config.display.output` argument of `crate::display::write_output` in `execute` with `OutputFormat::JsonCompact`, then run `cpu-guard --cores 2 -- cargo test --test cli complete_multilanguage_reports`: exit 101, the human whole-output assertion fails. Restore, replace it with `OutputFormat::Human`, rerun: exit 101, JSON parsing fails. Restore, replace `if config.analysis.fail_on_error && results.skipped_entries() > 0` with `if false && results.skipped_entries() > 0`, and run `cpu-guard --cores 2 -- cargo test --test cli unreadable_file_reports_output_and_fail_on_error_status`: exit 101, status 0 instead of 1. Restore after each; these are injected faults, not product defects. The environment above applies to each command.

After restoring production and using `CARGO_TARGET_DIR="$PWD/.work/cli-ordinary-target"`, `cpu-guard --cores 2 -- make check` exited 0 (507 unit, 29 CLI, 1 fixture); `cpu-guard --cores 2 -- make lint` exited 0 (Clippy and Ruff). Set `TMPDIR`, `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, `RUSTC_WRAPPER=kache`, and `PYTHON_CPU_COUNT=2` as above. An initial lint failed on `from_mode(0)`; changing its literal to `0o000` fixed Clippy, and the final mutation run used that spelling.

Warm `make test` with `perf-gate-ready` immediately before and `perf-gate-ready -1` immediately after: base 3.27s, candidate 3.27s, each pre/test/post exit 0/0/0. The base readiness probe reported a hot process on an initial attempt and subsequently passed. These are single observations, not a performance claim. The base ran with its original test file and the candidate file was restored afterward. No broader coverage or CI qualification is claimed.
