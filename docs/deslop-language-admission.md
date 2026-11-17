# Filename admission and default detection

Base `0e8794497441026de8cdbb58bed77122e456dcf6`; Darwin arm64, rustc `1.102.0-nightly (5b7b37aef 2026-11-08)`, cargo-mutants 27.1.0, default features. Only tests changed. This bounded scenario selects the complete `BestMatches::consider`, `get_candidates`, `default_candidate`, and `detect_language` methods, including their calls to scoring and shebang handling; it does not mutate those implementations. The complete selection was listed before mutation. The historical four `langs.rs` survivor identities were not available in the tracked records or this worktree's scratch, so they cannot be reconciled to this inventory. Broader Tests remain open.

`admission_keeps_only_the_most_specific_patterns` asserts literal candidate names for an equal-specificity prefix/extension tie, a stronger prefix, a longer uppercase extension and prefix case. `admission_and_default_reach_the_counting_consumer` asserts C for a header without content, and four literal line counts through `analysis::bench::detect_and_count`; it also checks an unknown name is not counted. `detection_reports_uncertain_and_decisive_samples` asserts language and close-call values for real headers and unknown names. The shipped table has no `shebang_overrides: true` entry (`languages.json5`; `build/config.rs` defaults the boolean); public callers can nevertheless provide a `Language`, so this last test also clones the real Rust definition with the flag enabled and checks a Python shebang and the no-content fallback.

Before: 28 selected, 11 caught, 11 missed, six unviable, no timeout; exit 2. An intermediate candidate: 19 caught, three missed, six unviable; another: 21 caught, one missed, six unviable. Each baseline passed. The last candidate's full outcomes follow below. A caught mutant is not necessarily caught by a newly added test; unviable replacements are compiler errors, not runtime detection. The `None` default replacement, for example, fails on an unused `candidates` parameter under denied warnings.

Run from the worktree, with a distinct `OUT` for each run and no concurrent build in the tree. The slot acquisition fails closed; only the successful owner releases it. The mutation target is separate from ordinary verification. Total elapsed for all four scenario runs was under 20 minutes.

```sh
mkdir -p .work/tmp
mkdir /Users/josh/d/stuff/.work/tmp/mutation.lock || exit 1
slot=''
trap 'if [ -n "$slot" ]; then rmdir "$slot"; fi; rmdir /Users/josh/d/stuff/.work/tmp/mutation.lock' EXIT
for n in 1 2 3; do
  if mkdir "/Users/josh/d/stuff/.work/tmp/deslop-heavy-slots/$n" 2>/dev/null; then
    slot="/Users/josh/d/stuff/.work/tmp/deslop-heavy-slots/$n"; break
  fi
done
[ -n "$slot" ] || exit 75
export TMPDIR="$PWD/.work/tmp" CARGO_TARGET_DIR="$PWD/.work/tmp/mutation-target"
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 RUSTC_WRAPPER=kache PYTHON_CPU_COUNT=2
# Use --list instead of --in-place and remove execution options to list the same selection.
timeout -k 5 1200 cpu-guard --cores 2 -- /Users/josh/d/stuff/.work/tools/bin/cargo-mutants mutants --in-place --no-config --skip-calls-defaults=false --file 'src/langs/detection*.rs' --file 'src/langs/detection/patterns.rs' --re '(BestMatches::consider|get_candidates|default_candidate|detect_language)(\b| ->)' --baseline run --timeout 60 --build-timeout 300 --no-shuffle --output "$OUT" -- --lib
```

Three hand faults were applied separately under the mutation lock, with a fresh heavy slot per `cpu-guard --cores 2 -- cargo test --lib TEST`, and the candidate files restored and compared after each: excluding `.env.` from the prefix check made the admission test fail (`["Fish"]` versus `["Dotenv", "Fish"]`); filtering out all defaults made the consumer test fail (`None` versus `Some("C")`); forcing `close_call` false made the detection test fail (`false` versus `true`). Each test exited 101 on its assertion, not at compilation. The full mutation selection independently exercises the shebang guard and the threshold example.

Ordinary verification uses `TMPDIR="$PWD/.work/tmp" CARGO_TARGET_DIR="$PWD/.work/tmp/ordinary-target" CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 RUSTC_WRAPPER=kache PYTHON_CPU_COUNT=2 UV_OFFLINE=1`, a separately acquired heavy slot per command, and `cpu-guard --cores 2 -- make check` and `cpu-guard --cores 2 -- make lint`. No production code changed; whole-step before/after timing is not established here.

The final candidate run exited 0: 22 caught, zero missed, six unviable, zero timeout. The complete before → final inventory follows. Line and column positions refer to the unchanged production source.

| Mutation | Before | Final |
| --- | --- | --- |
| `src/langs/detection.rs:14:75: replace == with != in default_candidate` | caught | caught |
| `src/langs/detection.rs:16:3: delete match arm (Some(lang), None) in default_candidate` | caught | caught |
| `src/langs/detection.rs:38:8: replace match guard candidates[0].shebang_overrides with true in detect_language` | caught | caught |
| `src/langs/detection.rs:40:8: replace match guard candidates[0].require_keywords != RequireKeywords::Always with true in detect_language` | caught | caught |
| `src/langs/detection.rs:40:8: replace match guard candidates[0].require_keywords != RequireKeywords::Always with false in detect_language` | caught | caught |
| `src/langs/detection.rs:40:39: replace != with == in detect_language` | caught | caught |
| `src/langs/detection/patterns.rs:13:18: replace > with == in BestMatches::consider` | caught | caught |
| `src/langs/detection/patterns.rs:13:18: replace > with < in BestMatches::consider` | caught | caught |
| `src/langs/detection/patterns.rs:17:18: replace == with != in BestMatches::consider` | caught | caught |
| `src/langs/detection/patterns.rs:38:30: replace + with - in get_candidates` | caught | caught |
| `src/langs/detection/patterns.rs:38:30: replace + with * in get_candidates` | caught | caught |
| `src/langs/detection.rs:36:3: delete match arm 0 in detect_language` | missed | caught |
| `src/langs/detection.rs:38:8: replace match guard candidates[0].shebang_overrides with false in detect_language` | missed | caught |
| `src/langs/detection.rs:54:40: replace || with && in detect_language` | missed | caught |
| `src/langs/detection.rs:54:69: replace != with == in detect_language` | missed | caught |
| `src/langs/detection.rs:55:85: replace && with || in detect_language` | missed | caught |
| `src/langs/detection.rs:55:67: replace < with == in detect_language` | missed | caught |
| `src/langs/detection.rs:55:67: replace < with > in detect_language` | missed | caught |
| `src/langs/detection.rs:55:67: replace < with <= in detect_language` | missed | caught |
| `src/langs/detection/patterns.rs:13:18: replace > with >= in BestMatches::consider` | missed | caught |
| `src/langs/detection/patterns.rs:41:34: replace + with - in get_candidates` | missed | caught |
| `src/langs/detection/patterns.rs:41:34: replace + with * in get_candidates` | missed | caught |
| `src/langs/detection.rs:14:2: replace default_candidate -> Option<&'static Language> with None` | unviable | unviable |
| `src/langs/detection.rs:14:2: replace default_candidate -> Option<&'static Language> with Some(Box::leak(Box::new(Default::default())))` | unviable | unviable |
| `src/langs/detection.rs:34:2: replace detect_language -> Detection with Default::default()` | unviable | unviable |
| `src/langs/detection/patterns.rs:13:3: replace BestMatches::consider with ()` | unviable | unviable |
| `src/langs/detection/patterns.rs:27:2: replace get_candidates -> Vec<&'static Language> with vec![]` | unviable | unviable |
| `src/langs/detection/patterns.rs:27:2: replace get_candidates -> Vec<&'static Language> with vec![Box::leak(Box::new(Default::default()))]` | unviable | unviable |

To reproduce a hand fault, acquire the mutation lock and a heavy slot as above. Save both source files and install a restoration trap before changing them. Apply one substitution at a time, run the named test with the exported mutation environment and `cpu-guard --cores 2 -- cargo test --lib TEST`, then restore and compare the file before the next fault. Release the owned heavy slot and acquire a new one for each test.

| File | Original → fault | Test |
| --- | --- | --- |
| `src/langs/detection/patterns.rs` | `filename.starts_with(prefix)` → `filename.starts_with(prefix) && prefix != ".env."` | `admission_keeps_only_the_most_specific_patterns` |
| `src/langs/detection.rs` | `lang.require_keywords == RequireKeywords::Never` → `{ let _ = lang; false }` | `admission_and_default_reach_the_counting_consumer` |
| `src/langs/detection.rs` | `close_call: margin < DECISIVE_MARGIN && surprising` → `close_call: margin < DECISIVE_MARGIN && surprising && false` | `detection_reports_uncertain_and_decisive_samples` |

The first final `make check` failed Clippy's `items_after_statements` lint. Moving the `OnceLock` declaration to the beginning of the test fixed it without changing its inputs or assertions. The final `make check` exited 0 with 512 unit, 29 CLI and one fixture test passing. The final `make lint` exited 0 with Clippy and Ruff passing. The override-guard mutant's final log shows the test assertion failing with Rust instead of Python. The production portions of both detection files match the base; only tests in `detection.rs` and `langs.rs` changed.
