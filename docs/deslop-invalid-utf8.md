# Invalid UTF-8 counting evidence

The counting test preserves first-line semantics when invalid UTF-8 forces
line-by-line decoding. Four independent Python inputs cover a UTF-8 BOM,
invalid bytes on the first or second line, and valid or unrelated interpreter
names on later lines. Each asserts all five line totals and zero embedded
lines. A later valid interpreter line remains a comment.

This is one Tests component. DSLP-4 and the broader counting scope remain open.
Runtime code, dependencies, public interfaces and CLI output are unchanged.

## Source and selection

Base: `dd84c9c49e7895244bbec6cfc4ffbd243806c4c3`. The final
`src/analysis/line_counter.rs` SHA-256 is
`f8db9d45caddc714726706aef7f0ed07f77028a682605641c4b34beda74c0d39`.
Its production prefix before `#[cfg(test)]` hashes identically on both sides:
`5be3b2db193a0b3462f3570c15efb845bec8d22cea351a9f28bb2f99c9fe71d9`.
The change adds one unit test and alters no executable production statements.

Tools: pinned nightly `nightly-2026-11-09`, cargo-mutants 27.1.0, native macOS
arm64, default features. Set repo-owned `TMPDIR`, `RUSTC_WRAPPER=kache`,
`CARGO_BUILD_JOBS=1` and `RUST_TEST_THREADS=1`. Use distinct absolute
`CARGO_TARGET_DIR` values for mutation and ordinary verification. Acquire the
campaign's exclusive mutation lock before running:

```sh
cargo mutants --no-config --skip-calls-defaults=false \
  --file src/analysis/line_counter.rs -F 'count_lines([^_a-z]|$)' --list --json
timeout -k 5 1200 cpu-guard --cores 2 -- cargo mutants --in-place \
  --no-config --skip-calls-defaults=false --file src/analysis/line_counter.rs \
  -F 'count_lines([^_a-z]|$)' --baseline run --timeout 60 --build-timeout 300 \
  --output .work/count-result -- --lib
```

The inventory contains exactly three identities, all in `count_lines`.
Before the test: two missed, one unviable, zero caught or timed out, exit 2.
After: two caught, one unviable, zero missed or timed out, exit 0. Each run
passes a fresh baseline and takes about 15 seconds under the two-core cap.
The unviable return replacement requires `Default` for `FileCounts`, which
does not implement it.

Both caught logs fail the new literal assertion. Flipping the first-line
decode condition produces three code lines and zero comments. Flipping the
classifier condition produces zero shebangs and three comments. These are the
two saved survivors at lines 28 and 29 of the historical counting selection.
This three-identity run does not requalify that entire 229-identity selection.

## Fault and ordinary verification

A separate hand fault marks every decoded line as the first line. The focused
test exits 101 with two shebangs instead of one. Restore the source before
running `cargo test --lib invalid_utf8_keeps_first_line_and_bom_rules`.
The earlier valid-interpreter-only test data allowed a first-line inversion
to swap two classifications without changing the totals. The final table
retains unrelated-interpreter cases as well, and the complete selection above
was repeated after that correction.

One ordinary run overlapped an in-place mutation and compiled changed source.
That run is discarded. Mutation and ordinary checks now run sequentially.
The final ordinary target was fresh, the production prefix matched the base,
and restored `make test` and `make lint` both exited 0. The suite passes 504
unit, 26 CLI and one fixture test, with no ignored cases.

After warming outside measurement, run `perf-gate-ready`,
`/usr/bin/time -p cpu-guard --cores 2 -- make test`, then immediately
`perf-gate-ready -1`. Both pre/test/post triples exit 0. Warm cost is 3.24 s
before and 3.27 s after, within the allowance. These single observations do
not establish a speed change. Production performance and consumer behavior
are unaffected by this test-only change. Linux CI remains a separate check.
