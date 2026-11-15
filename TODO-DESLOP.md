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

## Tickets


### DSLP-6 · P1 · Overlapping standard-input paths count files more than once

- **Area:** analysis · file selection
- **Where:** src/analysis/list.rs:28, src/analysis/list.rs:79, src/analysis/list.rs:125, tests/cli.rs:313
- **Problem:** A list containing a directory and a file inside it reports the file twice: a built CLI run with one one-line Rust file returned file_count 2 and total lines 2. Only byte-identical list entries are deduplicated, then listed files and directories are counted independently; the existing test covers an exact repeated filename but no overlap. This silently inflates counts. Test-gap finding: include src/analysis/list.rs and src/analysis/walk.rs in critical mutation scope.
- **Recommendation:** Deduplicate overlapping listed paths and equivalent spellings while preserving the list filters and symlink behavior; cover overlapping directories and files through the CLI.
- **Fix with:** root-cause
- **Effort:** M



### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** test suites and CI
- **Problem:** Test-gap finding: calibration (2026-11-15) measured 93.0% line and 84.5% branch coverage; 19 of 73 viable sampled mutants survived, in src/cli/mod.rs (11), src/langs.rs (4), src/config_discovery.rs (2), and src/analysis.rs (2). A full run was estimated at 2.3 hours. The CLI suite lacks a complete human or JSON output contract and an unreadable-file --fail-on-error test (AGENTS.md Verification); these boundaries can regress despite partial output and error checks. No deslop mutation lane exists.
- **Recommendation:** Run the deslop Tests step with critical mutation scope covering counting (src/analysis/line_counter.rs, line_classifier.rs, embedded.rs), language detection (src/langs.rs and src/langs/detection/), file selection (src/analysis/walk.rs, walk/rules.rs, list.rs, pipeline.rs), and src/config_discovery.rs. Add boundary witnesses for the named gaps. Accept: no undispositioned survivor in critical code, the test cost budget holds, and the lane is green where the tools are mirrored.
- **Fix with:** deslop
- **Effort:** M


### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository
- **Problem:** No dead-code scan has run.
- **Recommendation:** Run the deslop Dead code step. Accept: tools clean or allowlisted with reasons; walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** M

