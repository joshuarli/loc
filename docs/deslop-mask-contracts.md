# Mask scanner qualification

Base `f5603d85b27d53464c1c9836e0a72b171efc25f3`; candidate includes `7a133f54a6ecd5a41ecaaa816391ae2f6d64cde9` and the subsequent test change in this document's commit. Darwin arm64, rustc nightly-2026-11-09, default features, cargo-mutants 27.1.0. `count_text` uses the scanner for clear code and, with changed marker bytes, open comments and strings; it uses line search when the marker set is not suited (`src/analysis/line_counter.rs:45-103`). NEON is active on this host. AVX2 requires x86 CI or a suitable host; this run cannot qualify it. Broader DSLP-4 remains open.

The two tests exercise marker transitions, whitespace and non-whitespace controls, blank/code/comment counts, unterminated tails and offsets around 16, 32, 64 and 128 bytes. The Rust consumer case exercises the real `count_lines` path. The threshold comparison (`suits`, line 58) is a performance choice: scanner and fallback both find markers and count lines, but no universal output-equivalence or performance claim follows from one witness. The literal Rust consumer case passed both with the original threshold and with `<=` changed to `>`; the latter is the sole native miss. Its performance effect is unmeasured.

Both full runs used the same 97-entry inventory, SHA-256 `86b9e7e40e54bad0984937360e0fe054e861d496f936745559e411640346b5b8`. Base: exit 3, 57 caught, 14 missed, 17 unviable, nine timeouts. Candidate: exit 3, 58 caught, 13 missed, 17 unviable, nine timeouts. The only changed outcome is line 229 `-` to `+`, caught by the candidate consumer test: it reclassifies `0x0e` and `0x1f` as blank. The 12 x86-only misses at lines 96 and 179-204 are compiled out on ARM, not equivalent or killed. The 17 unviable mutants did not build and were not runtime tested. All identities are listed below; positions are unchanged between the runs.

The nine timeout identities each repeated with a 65-second limit. Per-case logs showed the same named test still running, and the process snapshots showed 54-63 seconds CPU during 55-64 seconds elapsed. Stack samples showed the counting caller or scanner active; these are actual prolonged loops under the mutated program, not CPU-guard interruptions. The bounded observation does not establish mathematical nontermination. The two interrupted one-core attempts in the earlier record are rejected and excluded from these counts.

The tail fault `masks.candidates &= 0` failed the new boundary test (exit 101; `(37, false)` versus `(23, true)`). Changing the ARM whitespace bound from `b'\r' - b'\t'` to `b'\r' + b'\t'` failed the consumer test (exit 101; code 4/blank 4 versus code 6/blank 2). Inverting `suits` passed the same consumer test (exit 0). All three controls acquired the lock, restored source between faults, and verified restoration with `cmp`. A prior tail fault accidentally ran without owning the lock after acquisition failed; it is rejected, and the locked rerun above replaces it. No other build or mutation ran in this worktree during that earlier fault.

`make check` exited 0 with 509 unit, 27 CLI and one fixture test, plus Clippy. No production code changed and no runtime benchmark is claimed.

## Caught (58)

```text
src/analysis/masks.rs:63:29: replace || with && in BlockScanner<'a>::load
src/analysis/masks.rs:63:20: replace != with == in BlockScanner<'a>::load
src/analysis/masks.rs:63:32: delete ! in BlockScanner<'a>::load
src/analysis/masks.rs:115:31: replace * with + in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:115:31: replace * with / in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:115:23: replace / with % in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:115:23: replace / with * in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:116:18: delete ! in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:116:32: replace - with + in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:120:49: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:120:49: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:120:81: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:120:81: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:121:11: delete ! in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:123:54: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:123:54: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:123:25: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:123:25: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:123:56: delete ! in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:124:13: replace != with == in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:125:11: replace += with -= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:125:11: replace += with *= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:126:21: replace += with -= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:126:21: replace += with *= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:126:31: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:126:31: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:128:35: replace + with * in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:128:19: replace + with * in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:50: replace != with == in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:21: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:21: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:23: delete ! in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:34: replace >> with << in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:41: replace - with + in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:129:41: replace - with / in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:131:14: replace |= with &= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:131:22: replace != with == in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:137:13: replace >= with < in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:138:14: replace < with == in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:138:14: replace < with > in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:138:14: replace < with <= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:139:12: replace += with -= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:139:12: replace += with *= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:140:22: replace += with -= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:140:22: replace += with *= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:146:52: replace - with + in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:146:52: replace - with / in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:155:3: delete ! in with_text_before
src/analysis/masks.rs:161:50: replace + with - in scan
src/analysis/masks.rs:161:50: replace + with * in scan
src/analysis/masks.rs:170:36: replace - with + in scan
src/analysis/masks.rs:170:36: replace - with / in scan
src/analysis/masks.rs:170:21: replace << with >> in scan
src/analysis/masks.rs:171:19: replace &= with |= in scan
src/analysis/masks.rs:172:13: replace &= with |= in scan
src/analysis/masks.rs:229:65: replace - with + in scan_block
src/analysis/masks.rs:229:65: replace - with / in scan_block
src/analysis/masks.rs:231:50: delete ! in scan_block
```

## Native performance-choice miss and x86 platform gaps (13)

```text
src/analysis/masks.rs:58:18: replace <= with > in BlockScanner<'a>::suits
src/analysis/masks.rs:96:3: replace BlockScanner<'a>::sweep_avx2 -> (usize, bool) with (0, true)
src/analysis/masks.rs:96:3: replace BlockScanner<'a>::sweep_avx2 -> (usize, bool) with (0, false)
src/analysis/masks.rs:96:3: replace BlockScanner<'a>::sweep_avx2 -> (usize, bool) with (1, true)
src/analysis/masks.rs:96:3: replace BlockScanner<'a>::sweep_avx2 -> (usize, bool) with (1, false)
src/analysis/masks.rs:179:2: replace scan_block -> Masks with Default::default()
src/analysis/masks.rs:186:4: replace | with & in scan_block
src/analysis/masks.rs:186:4: replace | with ^ in scan_block
src/analysis/masks.rs:186:61: replace << with >> in scan_block
src/analysis/masks.rs:196:65: delete ! in scan_block
src/analysis/masks.rs:203:2: replace whitespace -> __m256i with Default::default()
src/analysis/masks.rs:204:45: replace - with + in whitespace
src/analysis/masks.rs:204:45: replace - with / in whitespace
```

## Unviable, not runtime tested (17)

```text
src/analysis/masks.rs:58:3: replace BlockScanner<'a>::suits -> bool with true
src/analysis/masks.rs:58:3: replace BlockScanner<'a>::suits -> bool with false
src/analysis/masks.rs:63:3: replace BlockScanner<'a>::load with ()
src/analysis/masks.rs:80:3: replace BlockScanner<'a>::sweep -> (usize, bool) with (0, true)
src/analysis/masks.rs:80:3: replace BlockScanner<'a>::sweep -> (usize, bool) with (0, false)
src/analysis/masks.rs:80:3: replace BlockScanner<'a>::sweep -> (usize, bool) with (1, true)
src/analysis/masks.rs:80:3: replace BlockScanner<'a>::sweep -> (usize, bool) with (1, false)
src/analysis/masks.rs:111:3: replace BlockScanner<'a>::sweep_blocks -> (usize, bool) with (0, true)
src/analysis/masks.rs:111:3: replace BlockScanner<'a>::sweep_blocks -> (usize, bool) with (0, false)
src/analysis/masks.rs:111:3: replace BlockScanner<'a>::sweep_blocks -> (usize, bool) with (1, true)
src/analysis/masks.rs:111:3: replace BlockScanner<'a>::sweep_blocks -> (usize, bool) with (1, false)
src/analysis/masks.rs:155:2: replace with_text_before -> u64 with 0
src/analysis/masks.rs:155:2: replace with_text_before -> u64 with 1
src/analysis/masks.rs:161:2: replace scan -> Masks with Default::default()
src/analysis/masks.rs:214:2: replace scan_block -> Masks with Default::default()
src/analysis/masks.rs:244:2: replace bits -> u64 with 0
src/analysis/masks.rs:244:2: replace bits -> u64 with 1
```

## Repeated 65-second timeouts (9)

```text
src/analysis/masks.rs:116:24: replace << with >> in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:116:32: replace - with / in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:120:68: replace & with | in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:120:68: replace & with ^ in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:128:35: replace + with - in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:128:19: replace + with - in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:133:18: replace != with == in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:136:10: replace += with -= in BlockScanner<'a>::sweep_blocks
src/analysis/masks.rs:136:10: replace += with *= in BlockScanner<'a>::sweep_blocks
```

## Timeout witnesses

The table gives each mutation and the test that had not completed, the last observed elapsed and CPU time in the 65-second run, and the sampled caller location. `LC` is `src/analysis/line_counter.rs`. The first two locations come from a second 65-second run; the remaining samples come from the first longer run.

| Mutation | Test | Elapsed / CPU | Sample |
| --- | --- | --- | --- |
| 116:24 `<<` to `>>` | `assembly_directives_stay_code_when_swept` | 59s / 57s | LC 64, 71 |
| 116:32 `-` to `/` | `count_bytes_matches_process_file` | 63s / 62s | LC 64, 71, 74 |
| 120:68 `&` to `\|` | `bom_is_not_part_of_the_first_line` | 57s / 57s | LC 64, 74, 79 |
| 120:68 `&` to `^` | `bom_is_not_part_of_the_first_line` | 62s / 61s | LC 64, 74, 79 |
| 128:35 `+` to `-` | `sweeping_matches_classifying_every_line` | 55s / 55s | LC 71 |
| 128:19 `+` to `-` | `rust_counting_preserves_results_when_scanner_choice_changes` | 60s / 59s | LC 79 |
| 133:18 `!=` to `==` | `bom_is_not_part_of_the_first_line` | 64s / 63s | LC 79 |
| 136:10 `+=` to `-=` | `count_bytes_matches_process_file` | 57s / 57s | LC 64 |
| 136:10 `+=` to `*=` | `assembly_directives_stay_code_when_swept` | 62s / 61s | LC 64 |

## Reproduction

The full runs took about 695 and 714 seconds respectively. The following commands require an idle worktree and an absent parent quiet reservation. The lock acquisition must succeed before any in-place edit. The trap restores the candidate and releases only the acquired lock. Replace the output directory names if they already exist.

```sh
mkdir -p .work/tmp
export TMPDIR="$PWD/.work/tmp" CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 RUSTC_WRAPPER=kache
cpu-guard --cores 2 -- make check
mkdir /Users/josh/d/stuff/.work/tmp/mutation.lock || exit 1
cp src/analysis/masks.rs .work/tmp/candidate.rs || { rmdir /Users/josh/d/stuff/.work/tmp/mutation.lock; exit 1; }
trap 'cp .work/tmp/candidate.rs src/analysis/masks.rs; rmdir /Users/josh/d/stuff/.work/tmp/mutation.lock' EXIT

git show f5603d85b27d53464c1c9836e0a72b171efc25f3:src/analysis/masks.rs > src/analysis/masks.rs
/Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --no-config --file src/analysis/masks.rs --list > .work/tmp/before-list.txt
timeout -k 5 1200 cpu-guard --cores 2 -- /Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --no-config --file src/analysis/masks.rs --in-place --baseline run --timeout 30 --build-timeout 180 --no-shuffle --output .work/tmp/before --no-times -- --lib
cp .work/tmp/candidate.rs src/analysis/masks.rs
/Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --no-config --file src/analysis/masks.rs --list > .work/tmp/after-list.txt
cmp .work/tmp/before-list.txt .work/tmp/after-list.txt
timeout -k 5 1200 cpu-guard --cores 2 -- /Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --no-config --file src/analysis/masks.rs --in-place --baseline run --timeout 30 --build-timeout 180 --no-shuffle --output .work/tmp/after --no-times -- --lib
python3 - <<'PY'
import re
lines = open('.work/tmp/after/mutants.out/timeout.txt').read().splitlines()
open('.work/tmp/timeout-re.txt', 'w').write('|'.join('^' + re.escape(line).replace('\\ ', ' ') + '$' for line in lines))
PY
/Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --no-config --file src/analysis/masks.rs --re "$(cat .work/tmp/timeout-re.txt)" --list > .work/tmp/timeout-list.txt
cmp .work/tmp/after/mutants.out/timeout.txt .work/tmp/timeout-list.txt
timeout -k 5 1200 cpu-guard --cores 2 -- /Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --no-config --file src/analysis/masks.rs --re "$(cat .work/tmp/timeout-re.txt)" --in-place --baseline skip --timeout 65 --build-timeout 180 --no-shuffle --output .work/tmp/long --no-times -- --lib
```

The full and longer mutation commands exit 3 when timeouts remain. For loop evidence, inspect each test log under `.work/tmp/long/mutants.out/log`, and while its test process runs, collect `ps -axo pid,etime,time,command` snapshots and `sample <pid> 1 1 -file .work/tmp/sample-<pid>.txt`. The two additional stack witnesses used the same longer command with `--re ':(116:24: replace << with >>|116:32: replace - with /)'` and a fresh output directory.

Under the same lock and restoration trap, run the controls individually. Restore `.work/tmp/candidate.rs` after each edit. The fault commands below exited 101, 101 and 0 respectively.

```sh
python3 -c 'p="src/analysis/masks.rs";s=open(p).read();assert s.count("masks.candidates &= wanted;")==1;open(p,"w").write(s.replace("masks.candidates &= wanted;","masks.candidates &= 0;"))'
cpu-guard --cores 2 -- cargo test --lib sweep_counts_tails_and_stops_on_markers_at_byte_boundaries
cp .work/tmp/candidate.rs src/analysis/masks.rs
python3 -c 'p="src/analysis/masks.rs";s=open(p).read();a="vdupq_n_u8(b\x27\\r\x27 - b\x27\\t\x27)";b="vdupq_n_u8(b\x27\\r\x27 + b\x27\\t\x27)";assert s.count(a)==1;open(p,"w").write(s.replace(a,b))'
cpu-guard --cores 2 -- cargo test --lib rust_counting_preserves_results_when_scanner_choice_changes
cp .work/tmp/candidate.rs src/analysis/masks.rs
python3 -c 'p="src/analysis/masks.rs";s=open(p).read();assert s.count("searched.len() <= MOST_SEARCHED")==1;open(p,"w").write(s.replace("searched.len() <= MOST_SEARCHED","searched.len() > MOST_SEARCHED"))'
cpu-guard --cores 2 -- cargo test --lib rust_counting_preserves_results_when_scanner_choice_changes
cp .work/tmp/candidate.rs src/analysis/masks.rs
cmp src/analysis/masks.rs .work/tmp/candidate.rs
```
