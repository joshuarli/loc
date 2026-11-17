# Historical mutation sample rerun

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
