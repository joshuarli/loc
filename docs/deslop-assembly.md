# Assembly directive classification proof

Base `dd84c9c49e7895244bbec6cfc4ffbd243806c4c3`; SHA-256 of base `src/analysis/line_classifier.rs` is `06b1c9dd0e168f04c0192d77ddaed587edd534b042acd9dd3dc2d3b913e06e4f`, and of `languages.json5` is `745ca117caa903742a03533811d584c20d684829d08ec935d4a978dbe6b8b4da`. Candidate classifier SHA-256 is `a1fc551c49e1e83d794dcc5f678af7db22a4c1253dfddcc2f80e02e50146fd2f`; its only changes are in the test module. Assembly is the only `c_preprocessor` language in the table; its `# ` and `#\t` comments reach `classify_line`, which checks directives in clear state before comment matching. The new literal oracle covers directives and prose comments through that public classifier; it does not specify a complete preprocessor grammar.

On Darwin arm64, pinned rustc 1.102.0-nightly (5b7b37aef 2026-11-08), cargo-mutants 27.1.0, default features: each run used this selection, with `OUT` set to `.work/asm-before` or `.work/asm-after`:

```sh
export TMPDIR="$PWD/.work/tmp" RUST_TEST_THREADS=1 CARGO_BUILD_JOBS=1 RUSTC_WRAPPER=kache
timeout -k 5 1200 cpu-guard --cores 2 -- /Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --in-place --no-config --skip-calls-defaults=false --file src/analysis/line_classifier.rs --re 'is_preprocessor_directive( -> bool)?( with (true|false))?$' --baseline run --timeout 60 --build-timeout 300 --no-shuffle --output "$OUT" -- --lib preprocessor
```

The actual runs placed the timeout outside a bounded 120-second acquisition loop for atomic directory `/Users/josh/d/stuff/.work/tmp/mutation.lock`, releasing it only from the acquiring process. One mutant ran at a time. Baselines passed; before: exit 2, 14 selected, 10 caught, 2 missed, 2 unviable, 0 timeouts, 52s; after: exit 0, 14 selected, 12 caught, 0 missed, 2 unviable, 0 timeouts, 50s. The complete identities, at unchanged source positions, are:

- `228:2` replace function `-> bool` with `true` or `false`: both unviable before and after, because the compiler rejects unused parameter `line`; neither is counted as detection.
- Caught before and after: `232:64` `&&` to `||`, `232:37` delete `!`, `232:69` `!=` to `==`; `235:3`, `236:3`, `237:3` delete the respective define-family, else, and if/elif match arms; `240:5` and `240:63` `||` to `&&`; `240:90` and `240:102` `==` to `!=`.
- Previously missed `240:85` and `240:97` `||` to `&&`: both caught after. Their candidate logs `src__analysis__line_classifier.rs_line_240_col_85.log` and `..._col_97.log` show `assembly_preprocessor_conditions_and_comments` failing its literal assertion: respectively `[Comment, Code, Comment, ...]` and `[Comment, Comment, Code, ...]` instead of `[Code, Code, Code, ...]`.

A hand fault removing `elif` from the match arm was tested under the same lock with `timeout -k 5 1200`, `cpu-guard --cores 2`, the same environment and fresh `CARGO_TARGET_DIR="$PWD/.work/asm-ordinary-target"`, using `cargo test --lib assembly_preprocessor_conditions_and_comments`. It exited 101 at the new assertion: both elif lines became comments. The source was restored, and the candidate hash above was checked before ordinary tests. Mutation and ordinary commands did not overlap.

Using that separate target, `timeout -k 5 1200 env TMPDIR="$PWD/.work/tmp" CARGO_TARGET_DIR="$PWD/.work/asm-ordinary-target" RUST_TEST_THREADS=1 CARGO_BUILD_JOBS=1 RUSTC_WRAPPER=kache cpu-guard --cores 2 -- make test` exited 0: 504 unit, 26 CLI, 1 fixture, zero failures. The analogous `make lint` exited 0 (Clippy and Ruff). A warm timed repeat of each, with `perf-gate-ready` immediately before and `perf-gate-ready -1` immediately after, had pre/test/post exits 0/0/0: test 3.27s, lint 1.17s. The parent supplied the matched base warm observation 3.24s with gate exits 0/0/0 and 503 unit, 26 CLI, 1 fixture; the increase is 0.03s, below 20% or 30s. These are single observations, not a median.

The initial mutation command incorrectly combined `--in-place` with `--jobs 1` and exited 1 before mutation; removing that incompatible option produced the runs above. This is one bounded function scenario, not qualification of a CI mutation lane or completion of the wider Tests component. No release benchmark was run because production behavior and runtime code are unchanged.
