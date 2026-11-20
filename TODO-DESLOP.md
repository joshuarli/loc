Prefix: DSLP
Next: 7

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier Calibration). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Ground: base `9ea3b7a521a3a6e8d1d95d0623080edd4fa51573`; `make test` cold 10.16s, warm 1.56s; `make lint` under 4s. PR CI runs `make test`; no missing-tool test skips found. Performance workload: `count_generated_corpus` in `benches/bench.rs`, baseline `benches/baseline.json`. Open fleet lint findings: rust-collections 12, linux-musl-toolchain 4, audit-upgrades 2, binary-release-static 1, make-clean-target 1, owned-tool-support 1, registry-sources 1, topics 1.

Types (DSLP-3): escape-hatch matching lines 326 → 326; no settings adopted. Unique findings below include production and bench targets, excluding tests for unwrap/expect. Real means demonstrated defect or removable trust; FP means intentional/justified use. Zero-finding settings lack evidence for adoption.

| Setting | Findings | Real / removable trust | FP | Decision |
| --- | ---: | ---: | ---: | --- |
| unsafe_code forbid | 5 | 0 | 5 | Reject: mmap and SIMD require unsafe on this host. |
| wildcard_enum_match_arm | 7 | 0 | 7 | Reject: guarded fallbacks, non-exhaustive enum, and test matches. |
| match_wildcard_for_single_variants | 0 | 0 | 0 | Reject: no witness. |
| cast_possible_truncation | 0 | 0 | 0 | Reject: no witness. |
| cast_sign_loss | 0 | 0 | 0 | Reject: no witness. |
| unwrap_used outside tests | 15 | 0 | 15 | Reject: benchmark setup and measurements. |
| expect_used outside tests | 13 | 2 | 11 | Reject: invariant and failure checks exceed FP budget; two block-comment assertions could carry more precise inputs. |
| redundant_clone | 0 | 0 | 0 | Reject: no witness. |
| needless_pass_by_value | 0 | 0 | 0 | Reject: no witness. |

Tests at `897131f` recovered all 19 historical missed identities and the
historical timeout. Configuration, CLI dispatch and language-list tests catch
13 of those 20 in the current source. The [mutation record](docs/deslop-historical-tests.md)
accounts for the remaining five misses and two confirmed timeouts. The wider
critical inventory, suite-cost and hosted whole-change qualification remain open.

Tests inventory at `341cbeef5a07088ff33153d0244db01443178c02`: CLI parsing, help, version and exit statuses are guarded by `tests/cli.rs` and `src/cli/command.rs`; complete human and JSON reports by `complete_multilanguage_reports`, schema by `schema_has_one_shape_for_counts_at_every_level`; configuration precedence by `configuration_discovery_prefers_local_then_platform_then_home`, merge and init by their unit tests. File selection, stdin, aliases, filters and git/diff have CLI witnesses; line classification and languages have `tests/fixture_counts.rs`, unit tests and LineJudge. Public Rust modules and `run` have in-process coverage but no external consumer contract test; known consumers are the binary and `benches/bench.rs`, others unknown. Flag combinations are not exhaustive. No network API exists. Acceptance: `make test`, `make lint`; native macOS ARM and Linux ARM/x86 musl are supported, Windows unsupported.

Test tiers: unit parsing and classification (Edit), binary CLI and fixture tests (Change), LineJudge and native platform jobs (Scenario); no separate Full suite is declared. Ground measured warm `make test` 1.56s at `9ea3b7a`; test-runner totals in the current fresh build were 0.74s unit, 0.76s CLI and 0.03s fixtures. These are not a paired whole-change cost.

Critical decisions: detection and shebang, string/comment/blank classification, marker scanning and tails, file admission and deduplication, config precedence, dispatch and error status. Historical inventories and exact identities live in `docs/deslop-historical-tests.md`, `docs/deslop-counting-contracts.md`, `docs/deslop-mask-contracts.md`, `docs/deslop-language-admission.md`, `docs/deslop-assembly.md`, `docs/deslop-invalid-utf8.md`, and `docs/deslop-cli-output.md`. Their counts are scoped runs, not a whole-suite mutation score. New tests and their injected faults were established in those records; this step adds none.

- Equivalent: three historical dispatch comparisons return the same root selection for the registered commands; the search refresh `<`/`<=` difference only repeats a search at equality.
- Accepted risk: the native `suits` threshold survivor changes scanner selection; output equivalence is supported only by the recorded witness and performance is unmeasured. Two encoding sample-length survivors affect the benchmark helper and are unqualified. The hosted uncovered-branch control had three survivors; this advisory scope is not a retained guarantee.
- Untested: wider file-selection and SIMD decision inventory, unviable mutations, and the four historical language-list identities that could not be reconciled to the later inventory. Nine mask-sweep mutants and two search-sweep mutants repeatedly timed out under mutation; they exhibit prolonged progress failures, not proven termination results. Historical dispatch hangs were subsequently caught with a child deadline.
- Needs environment: 12 x86-only mask mutants were compiled out on native ARM; broader ARM and x86 SIMD mutation coverage is unrun. The hosted native mixed-union repeat and paired suite-cost comparison remain unrun (the prior mixed control hit 600s and the local union passed); hosted billing and a quiet host are needed. Linux musl runtime and hosted repeat are not rerun in this local step. Coverage is advisory; the local whole-repo line/branch measurement is pending. Existing mutation CI lanes remain unchanged; shebang and changed line-classifier decisions are retained, wider sweeps advisory. The recorded warm assembly slice rose 3.24s to 3.27s; a paired whole-change cost remains unavailable on the busy host.

Dead code: on macOS ARM default features, `make lint` compiled all targets with warnings denied and cargo-machete 0.9.2 found no unused dependencies. No dependency was disputed, so udeps was not needed. The binary enters through `src/main.rs` and the library exports modules in `src/lib.rs`; public consumers beyond the binary and benchmark are unknown, so public exports cannot be cut on a local reference count. The walker, scanner, file list and CLI dispatch serve declared behavior; no behavior-preserving removal was demonstrated in the debloat inspection. Zero lines and zero dependencies removed; no allowlist. Linux musl-specific compilation remains hosted evidence, Windows is unsupported. No runtime or consumer contract changed.

## Accepted findings

DSLP-6 is fixed in [#11](https://github.com/joshuarli/loc/pull/11). Listed files
and directory walks share counted canonical paths after selection. The CLI
regression fails before the repair (four files instead of two) and passes for
overlaps, path spellings, NUL lists, exclusions, depth, symlink aliases and 300
aliases across worker batches. Local `cpu-guard --cores 6 -- make test lint`
passes; macOS and both Linux musl CI jobs pass at code head `7618617`.

Five interleaved pairs of ten dist-profile CLI processes on the 800-file
generated benchmark corpus have identical output and no difference exceeding
both the observed spread and 2%. Walk medians are 15.484/16.344 ms;
unique-stdin medians are 6.604/6.680 ms (baseline/candidate). Gate checks
returned zero before and after both workloads. Both binaries are 1,946,672
bytes; five walk maximum-RSS samples overlap (4,653,056–4,669,440 before,
4,587,520–4,685,824 after). Full method and evidence are in the PR.

## Tickets

