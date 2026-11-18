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



### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** CLI reporting, src/analysis counting and file selection, src/langs and detection, src/config_discovery.rs, tests and hosted CI
- **Problem:** The CLI tests now pin human and JSON output and unreadable-file exit handling. Both former historical hang mutants are caught with the child deadline. Linux XDG precedence runs on both hosted musl targets. Historical mutation and coverage counts are scoped in docs/deslop-historical-tests.md, docs/deslop-mask-contracts.md and docs/deslop-counting-contracts.md.
- **Recommendation:** Finish the counting, language detection, walk/list and configuration decision inventory. Retain the ARM and x86 SIMD dispositions, separate encoding benchmark-helper decisions, and verify the actual whole-change lane, runner cost and hosted controls on macOS and both musl targets.
- **Fix with:** deslop
- **Effort:** M


### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository
- **Problem:** No dead-code scan has run.
- **Recommendation:** Run the deslop Dead code step. Accept: tools clean or allowlisted with reasons; walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** M

