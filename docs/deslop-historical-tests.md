# Historical mutation sample rerun

At `ac9ffed`, the root-option CLI witness now kills and reaps a stalled
process after ten seconds. Cargo-mutants 27.1.0 singleton shards `15/40`
and `18/40` recaptured the two historical dispatch hangs as assertion
failures. Use `--no-config --file src/cli/mod.rs --sharding round-robin`
and pass `--test cli root_options_with_values_preserve_subcommand_dispatch`
after `--`. Each shard listed exactly one identity and reported one
caught, zero missed and zero timeout. This rerun covers the focused
dispatch case. The whole-change gate remains open.

At `897131f0948931480008d9b4363a4fb5bfc6ca85`, cargo-mutants 27.1.0 listed
all 19 earlier missed identities and the earlier `select_subcommand`
timeout at their original locations. The 2026-11-15 sample contained 53
caught, 19 missed, one timeout and 49 unviable outcomes. The rerun below
uses only the 20 recovered misses and timeout, on macOS arm64.

| Change | Caught | Missed | Timeout |
| --- | ---: | ---: | ---: |
| New configuration precedence witness | 2 | 16 | 2 |
| Add root-option dispatch and broken-pipe help witnesses | 8 | 10 | 2 |
| Rerun five selected survivors with stdin and language-list witnesses | 5 | 0 | 0 |

The final row reruns a subset of the preceding row. Across the recovered
identities, 13 are now caught, five still survive, and two hang. Both hangs
recurred at 60 and 90 seconds. At their locations, replacing `index += step`
with `index *= step` leaves `index` at 1 when the first option consumes one
argument. The hang remains an explicit outcome.

The three remaining comparison variants in `select_subcommand` change
whether a hyphen-prefixed token returns before the subcommand lookup.
No registered subcommand starts with a hyphen, and an unmatched token
returns `Selection::Root` immediately afterward. Those variants are
equivalent for the current command table. The two remaining
`bench::count_as` variants change the encoding sample length. They remain
unqualified in the benchmark helper.

Select the following identities by file, location and operation. Anchor
the escaped identity text with `--re` and use `--file` for these four files.
Check that `--list` matches the 20 entries before running. The final
five-identity rerun uses `short_option_step` at `190:21` and the four
`print_all_languages` identities.

| File | Locations and operations |
| --- | --- |
| `src/analysis.rs` | `44:57` replace `*` with `/` or `+` |
| `src/cli/mod.rs` | `113:30` replace `==` with `!=`. `148:27` replace `+=` with `-=` or `*=`. `148:46` replace `\|\|` with `&&`. `154:10` replace `+=` with `*=`. `157:60` replace `>` with `==`, `<` or `>=`. `190:21` replace `\|\|` with `&&`. `196:75` replace `==` with `!=`. `196:84` replace `-` with `/` or `+`. |
| `src/config_discovery.rs` | `19:2` replace `config_dir` with `None` or `Some(Default::default())` |
| `src/langs.rs` | `51:46` and `52:35` replace `+` with `*`. `52:48` replace `>` with `>=`. `59:16` replace `>` with `==`. |

Run in a disposable checkout. Set `TMPDIR` to `.work/tmp`,
`CARGO_NET_OFFLINE=true`, `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1` and
`RUSTC_WRAPPER=kache`. Use `cpu-guard --cores 2 -- cargo mutants --in-place
--no-config --baseline run --timeout 90 --build-timeout 180 --no-shuffle`
with the selected files and regex. Pass `-- --lib --test cli` to Cargo.

The ordinary candidate passed 513 library tests, 33 CLI cases and one
fixture test. Locked Clippy and pinned Ruff 0.17.3 passed. Ruff ran with
`--isolated` to avoid loading the enclosing Stuff checkout's config.
Linux execution, the wider critical inventory and the hosted gate remain open.

## Shebang classification

At `82742bd8`, the classifier treated the first line as a shebang when
its interpreter matched the selected language. A different interpreter
and a matching directive after the first line remain comments. The
Python classification cases exercise all three outcomes.

Cargo-mutants 27.1.0 selected 16 identities from the unchanged shebang
module, blob `2e8a5ae81f264fcb4f63f198827a023babe5409a`. The baseline
caught four, missed the outer `&&` in `is_shebang_for`, and had 11
unviable. The test catches that miss, with all five viable identities
caught and no timeouts. Both inventory files have SHA-256
`aa093ab30c14d0a3f92acc49591f8bd32f79498db7f0e80e2e4749f47beaf860`.

From a disposable checkout run `cargo mutants --no-config --file
src/langs/detection/shebang.rs --timeout 25 -- --locked`, with
`TMPDIR` under `.work/tmp`. This slice covers only shebang detection
and first-line classification.

## Hosted shebang mutation trial

The [positive hosted run](https://github.com/joshuarli/loc/actions/runs/44278389932)
at `0ee7305a` caught all five viable shebang mutations on macOS and
both Linux musl targets. Each run classified 11 builds as unviable.
The three hosted jobs took 380, 229 and 162 seconds respectively,
for 771 runner seconds. The Linux jobs ran Cargo tests with the
explicit musl targets; the ordinary musl container jobs and
LineJudge also passed.

| Control | Result on all three targets |
| --- | --- |
| [Documentation](https://github.com/joshuarli/loc/pull/25) | No mutations selected. |
| [Foreign-interpreter case removed](https://github.com/joshuarli/loc/pull/26) | The outer `is_shebang_for` disjunction survived. |
| [Removal and a preprocessor decision edit](https://github.com/joshuarli/loc/pull/27) | The same shebang mutation survived. Seven changed preprocessor identities were caught and two builds were unviable. |
| [Uncovered branch](https://github.com/joshuarli/loc/pull/28) | Three of six changed-region identities survived. |

Ordinary tests, lint and LineJudge passed all four controls. The
mixed gate took 261.3 seconds on x86-64 musl, 256.8 on ARM64 musl
and 595.4 on macOS. A test-only hunk inside a verified trailing
test module selects the protected shebang file. When a hunk changes
the same file's production code, the native diff adds those mutations.

Use Python 3.14, cargo-mutants 27.1.0 and ignored `.work/` scratch.
Run `make mutants-diff BASE=origin/master` from the trial branch.
On Linux, set `CARGO_BUILD_TARGET` to the installed native-architecture
musl target and set the matching Cargo target linker to `rust-lld`.
The broad ARM and x86 SIMD inventory and the paired suite cost remain
in DSLP-4.
