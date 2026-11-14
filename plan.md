# loc performance plan

Improve the existing counter, not its feature count. Keep the small text/JSON CLI,
make common source files cheaper to count, and accept a small, explicit accuracy
trade only when it buys a substantial measured speedup.

Source review: 2026-11-13, America/Los_Angeles. Repository checkpoint:
`ca69ed1ff5317baf9010e7799998f4a909021ab8`. This is a proposed implementation
campaign; no speedup or test result is claimed here. Reconcile with newer commits
before starting. Follow [AGENTS.md](AGENTS.md); the package is still `codestats`
and the executable is `cs`. Renaming either is outside this campaign.

## Scope and execution order

Use this file as the campaign's single checklist; do not create a second roadmap
or tracking framework. Mark an experiment accepted or rejected with a short
result and a reproducible evidence path. Rejected experiments are useful results;
remove their production code rather than accumulating alternate engines.

- [ ] P0 — Establish trustworthy parser, process, and real-tree baselines.
- [ ] P1 — Compact immutable scan plans and matcher storage.
- [ ] P2 — Reuse masks and candidate positions across the slow-path boundary.
- [ ] P3 — Compact lexical state and a few count-only scanner families.
- [ ] P4 — Measure NEON, AVX2, optional AVX-512, and scalar/SWAR variants.
- [ ] P5 — Reduce redundant encoding and byte-classification passes safely.
- [ ] P6 — Remove measured detection, I/O, scheduling, and aggregation overhead.
- [ ] P7 — Evaluate tightly bounded approximation, separately from exact work.
- [ ] P8 — Qualify the winning combination and delete losing machinery.

P0 comes first. After it, one parser owner sequences P1/P2/P3; another worker can
run isolated ISA experiments for P4, and an I/O worker can own P6. P5 starts after
the scan interface settles. P7 must be compared against the improved exact engine,
not a conveniently slow old baseline. Run performance measurements serially on a
quiet host; parallel implementation does not mean parallel benchmark jobs.

**Non-goals:** an analytics suite, MCP/server mode, themes, HTML reports, a full
parser or syntax highlighter, persistent caches/watchers, a new executor, a new
benchmark framework, or Windows support. Preserve the `lexopt` CLI and the
current dependency reductions. Do not reintroduce removed features to achieve
upstream parity. Keep new runtime dependencies at zero unless a measured,
documented trade makes one clearly preferable to existing code.

## What already exists — do not rebuild it

The reviewed source already has much of the machinery commonly proposed for a
fast counter:

| Existing mechanism | Source | Consequence for this plan |
| --- | --- | --- |
| 64-byte newline/candidate/non-whitespace masks, AVX2 and NEON, cached last block, bulk line accounting | [masks.rs](src/analysis/masks.rs) | Already adapted from Mezura. “Add SIMD” is not a new task. |
| Clear-state sweeps, sweeps inside eligible comments/strings, simple comment and quoted-line shortcuts | [line_counter.rs](src/analysis/line_counter.rs) | Optimize handoffs and residual work, not a replacement from scratch. |
| Lazy per-language matchers, small-byte-set `memchr` paths, token buckets and quick string checks | [matchers.rs](src/langs/matchers.rs) | Measure setup and representation costs before changing algorithms. |
| Cross-line comments, strings, interpolation, heredocs and custom string delimiters | [line_classifier.rs](src/analysis/line_classifier.rs) | Preserve state transitions when specializing common cases. |
| Reused thread-local read buffer; a 256 KiB mapping threshold; a 64-byte buffer offset | [file_io.rs](src/analysis/file_io.rs) | Tune existing policies rather than adding them again. |
| Filename filtering, sampled detection, wider ambiguous-file probing, `fstat` after opening | [pipeline.rs](src/analysis/pipeline.rs) | Find avoidable allocations and passes without silently changing the selected files. |
| Custom work-stealing walk with lazy worker startup and wakeups | [walk.rs](src/analysis/walk.rs) | No wholesale migration to Rayon, async I/O, or another walker. |
| Rustybench time/allocation harness, corpus scripts, LineBench and LineJudge configuration | [benches](benches/bench.rs), [scripts](scripts), [.linebench](.linebench), [.linejudge](.linejudge) | Extend these assets; do not replace them. |

The current masks already compute non-whitespace along with newlines and
candidates. Mezura's separate word/content classification is not automatically an
improvement. Its content-counting convention is also not this project's contract:
a line containing only `}` must not disappear into a new “extra” bucket.

The latest checkpoint has already replaced Clap with `lexopt` and moved developer
instructions into `AGENTS.md`. Keep those changes. Treat release-profile and
allocator choices as measured variables, not excuses to restore dependencies.

## Contracts and adoption rules

### Exact optimizations

Keep the existing default counting semantics. The local
[LineJudge dialect](.linejudge/dialects/codestats/default.toml) says whitespace-only
lines are blank even inside strings/comments, ordinary strings are code, and
eligible documentation strings count as comments. Preserve the actual
whitespace, shebang, line-ending and embedded-language conventions too.

An exact fast path must agree with the reference on both the counts and its
outgoing lexical state. Equal totals alone can hide a state error that corrupts
the next line. The existing classifier is a regression oracle, not infallible
language truth: independently confirmed bugs get reduced fixtures and separate
fixes rather than becoming permanent compatibility requirements.

For each selected top-level file, verify the physical-line partition:
`total = code + comment + blank + shebang`. Test the empty file, BOM-only input,
trailing and missing newline, CRLF and lone CR under the existing contract.
Embedded counts are a breakdown already included in their parent; never add them
to the physical total a second time. Preserve file inclusion, duplicate-path
handling, error behavior, output schema and stable report ordering.

No accuracy budget permits out-of-bounds reads, unchecked invalid UTF-8,
state-counter overflow, unbounded queues, quadratic rescanning, silently omitted
files, or swallowed I/O errors. Avoid arbitrary byte-range parallel parsing: a
worker cannot assume its range begins outside a string or comment.

### Performance and size

Freeze a baseline binary before each experiment. Use the same compiler, target,
profile, allocator and PGO status for before/after comparisons. Record scanner
bytes/second, instructions/byte where available, wall time, CPU time, allocations,
peak memory, and stripped distribution binary size. Scanner-only and end-to-end
wins must be labeled separately.

Suggested adoption thresholds, to calibrate against P0's measured noise:

- Prefer at least a reproducible 5% end-to-end improvement on an affected real
  workload, or a 15% parser improvement on a documented parser-heavy workload with
  no meaningful end-to-end regression. Simpler code or fewer allocations at
  equivalent speed can also win.
- Investigate a repeatable regression above 3% on a priority workload/architecture;
  do not dismiss it with a grand average. A binary-size or peak-memory increase
  above 5% needs a recorded compensating benefit, not an unnoticed baseline reset.
- No promised overall multiplier. Publish corpus-by-corpus results, including
  losses. A noisy result is inconclusive, not a pass or a reason to weaken gates.

These are experiment-selection rules, not performance claims. Do not spend weeks
forcing a marginal optimization past them; stop, record the result, and move on.

## P0 — Establish attribution and correctness baselines

**Files:** `benches/bench.rs`, `scripts/bench.sh`, `scripts/costs.py`,
`scripts/corpus.sh`, the existing conformance adapters, and narrow dev-only hooks
in the counting modules if needed.

### Extend, do not replace, the harness

The current four benchmark functions cover generated-tree analysis, fixture-tree
analysis, compact JSON rendering and human rendering. They do not isolate the
scanner. Add these distinct workloads:

1. In-memory counting of already-selected language/encoding, with fixture loading
   outside the measured closure. Separate first-use matcher construction from
   warmed scanning, reset file state every iteration, and consume all results.
2. In-memory detection plus counting through `pipeline::count_bytes`, to expose
   candidate selection, sampling and decoding costs independently of disk I/O.
3. Real-process empty directory, one file, tiny repository, and large repository;
   these include startup and first-use initialization that a warm library loop
   hides. Record output costs separately; do not compare a null sink to a terminal.
4. Read/walk/aggregate stages and allocation counters, instrumented only in a
   diagnostic build. Validate conclusions with uninstrumented dist binaries.
   Concurrent phase durations overlap; do not present their sum as wall time.

Use a small private/dev-only benchmarking seam if the scanner is not externally
reachable. Do not turn its internals into a supported public library API merely
to benchmark them. Reuse Rustybench; do not add Criterion or a telemetry system.

### Repair misleading comparisons

`scripts/costs.py` subtracts an absent-language run from each language's run.
But `pipeline::worth_reading` can reject files before opening/reading them. That
subtraction is a useful work estimate, not a clean parser-time measurement.
Keep its role explicit and use direct parser benchmarks or real phase attribution
for parser claims. A missing language in a candidate report must fail the
comparison, not become zero instructions and appear to be an improvement.

`scripts/bench.sh` pins revisions when it fetches a corpus, but reuses an existing
checkout without verifying its current revision. Check the actual commit and
worktree contents before measuring. Record every competitor's executable hash,
version/commit and build profile; do not depend on whatever happens to be on PATH.

Maintain two labeled external comparisons: matched work and each tool's defaults.
For matched work, verify the actual selected file manifest, total input bytes and
per-file physical lines before timing; an extension list or approximately equal
aggregate totals is insufficient. Explicitly align ignored/hidden/generated
files, attributes, configuration, language scope and embedded counting policy.
Use existing adapters where possible. Treat published upstream charts as leads,
not this fork's baseline. Never compare warm persistent-cache answers to full scans.

### Workload matrix

Use small checked-in fixtures and pinned real corpora from the existing scripts.
Prioritize Rust, C/C++, Python, JavaScript/TypeScript, shell, JSON/TOML/YAML, and
mixed HTML/Markdown/embedded code. Add deterministic synthetic axes rather than
only repeating the same Rust file 800 times:

- Many tiny files versus a few large files; shallow versus deep trees; dense
  ignore/attribute rules; mostly irrelevant files; tracked-file and stdin lists.
- Marker-free code, blank-heavy input, dense punctuation, long comments/strings,
  interpolation, heredocs, nested comments, minified single lines, and non-ASCII.
- Marker-set cardinality and candidate density independently; 0–128-byte inputs,
  lengths around SIMD boundaries, and files near each read/map threshold.

Primary platforms are Apple Silicon macOS, aarch64 Linux/musl, and x86-64
Linux/musl. Exercise AVX2-only x86 separately from AVX-512 when hardware exists.
Record CPU/features, OS/filesystem, compiler, build flags, worker count, power
mode, host load and thermal stability. Use counterbalanced A/B runs with a
control before/after, repeated samples and uncertainty. Report warm page-cache
and cold/first-run results separately. Do not label a run cold without controlling
or measuring cache state; do not globally drop caches on a shared host.

**Exit:** reproducible baseline artifacts, count/manifest checks, independently
measurable scanner work, and a ranked cost profile. Store scratch under `.work/`;
commit a compact evidence summary, not downloaded corpora or megabytes of logs.

## P1 — Compact immutable scan plans

**Files:** `src/langs/matchers.rs`, `src/langs/data.rs`, `build.rs`, `build/render.rs`.

Measure lazy matcher setup and hot lookup separately. Candidate experiments:

- Generate immutable primitive lexical descriptors from `languages.json5` using
  the existing generator. Keep one registry and a few scanner families; do not
  generate hundreds of duplicated parser bodies.
- Compare `TokenStarts`' 256 nested vectors with a compact offset table plus flat
  token slices. Preserve longest-token precedence and stable ties.
- Compare heap-owned `ByteSet` data with compact static slices/bitsets; deduplicate
  identical delimiter plans across languages only where profiling justifies it.
- Specialize one/two/three-marker cases and simple quote pairs using the existing
  `memchr` infrastructure. Retain the generic matcher for complicated cases.
  Remove `aho-corasick` only if all its uses are replaced with demonstrated wins;
  eliminating a dependency is not permission to regress dense-marker inputs.
- Cache dispatch decisions at file/scanner construction, not per byte or line.
  Benchmark a direct family match against function pointers: neither is free.

Do not replace one runtime initialization cost with huge generated tables and
instruction-cache pressure. Report build time, first-file latency, steady-state
speed, allocations and binary size. Do not add a code-generation dependency.

**Exit:** exact language/token dispatch, lower measured setup or scan cost, and
no unjustified expansion of generated code or duplicated language definitions.

## P2 — Keep the mask pass useful after it finds a marker

**Files:** `src/analysis/masks.rs`, `src/analysis/line_counter.rs`,
`src/analysis/line_classifier.rs`, `src/analysis/line_classifier/strings.rs`.

The current sweep finds a candidate-containing line; `line_end`, quick checks and
the classifier may then search that line again. Instrument these transitions
before assuming they dominate. Compare incrementally:

1. Carry already-known newline boundaries across the sweep/classifier handoff.
2. Expose a bounded cursor over candidate bits from the cached block so the
   classifier verifies actual tokens only at relevant positions.
3. Compare separate state-specific masks with a cached superset mask plus a small
   state-dependent filter. Avoid rescanning the same block whenever the active
   state changes, but measure the extra candidates introduced by a superset.
4. Where eligible, account for an entire plain code/comment/string run with
   newline and non-whitespace masks, including the partial line carried across
   a block boundary. Keep dense-marker and exotic-syntax fallback paths.

Important: `ByteSet::covering` may select punctuation *inside* a token rather than
its first byte. A quote can cover a prefixed raw-string opener. Candidate bits
are not token-start positions. Preserve the required lookbehind and precedence,
or use a separate true-start descriptor. False positives can be verified; false
negatives silently corrupt counts. State changes must not reuse masks that omit
newly relevant delimiters.

Keep cursor storage bounded to a block or small fixed window; do not materialize
per-byte kinds, token vectors or file-sized newline indexes. Preserve one-pass
progress on adversarial long lines and marker-dense input. If a quick path fails,
resume from a valid checkpoint rather than repeatedly restarting a long prefix.

**Exit:** exact counts *and state* against the existing path, reduced rescanned
bytes on the measured hot workloads, and a real scan/end-to-end gain. If reusing
only newline boundaries wins and a full event cursor does not, ship the smaller
change.

## P3 — Small common state; rare syntax stays cold

**Files:** the classifier, `strings.rs`, `embedded.rs`, and the scan-plan code.

Prototype a compact copyable common state: normal, line comment, block comment
with depth/pair, ordinary quoted string, and the minimal continuation information
needed by the selected family. Separate immutable language rules from mutable
per-file state. Use a line transition of conceptually
`(state, bytes) -> (line facts, next state)` for testing; aggregate directly in the
hot scanner rather than allocating line facts in production.

Keep heredoc terminators, custom delimiters and interpolation stacks in reusable
cold scratch or the existing generic state. `Vec::new()` does not itself allocate;
prove any allocation reduction instead of counting fields. Do not impose a small
nesting limit, truncate delimiters, saturate into a wrong state, or allocate a
large fixed stack in every worker to avoid a rare allocation. Spill/fallback must
retain complete state and consume every byte exactly once.

Specialize lexical *families*, not individual keyword vocabularies: C-like,
hash-comment, simple structured data, and generic/embedded as the initial set.
Add a family only if P0 shows substantial traffic or cost. The counter needs
code/comment/blank/shebang decisions, not identifiers, numbers, operators, ASTs
or rendered token spans.

Check save/restore at line boundaries against uninterrupted scanning, including
exiting a comment and finding code on the same line. A checkpoint is associated
with its language/plan; never restore it under another language. Arbitrary read
chunk boundaries require delimiter/UTF-8 carry handling as well—line checkpoints
alone do not make arbitrary chunk parallelism correct.

**Exit:** a smaller/hotter common state and a demonstrated win without two
independently maintained full parsers. Keep the generic implementation as a
fallback and test oracle, not a competing product surface.

## P4 — ISA and byte-class experiments, especially ARM

**Files:** `src/analysis/masks.rs`, `src/swar.rs`, relevant benchmark fixtures.

The current scanner already has AVX2/NEON and stops using block masks above eight
searched bytes. Sweep that cutoff against cardinality, density and line length;
do not assume one cutoff is ideal on every architecture.

- **NEON:** compare 16/32/64-byte processing, mask packing, instruction scheduling,
  and vector reuse between newline, candidates and whitespace. Measure Apple
  Silicon and Linux ARM independently. Optimize the common ARM path before
  making x86-only throughput the project's headline.
- **AVX2:** compare the current per-marker equality loop with small-set unrolling
  and table/shuffle-based membership for larger sets. An arbitrary byte set is
  not automatically representable by one collision-free nibble table; prove
  membership over all 256 byte values or explicitly verify false positives.
- **AVX-512:** optional 64-byte-mask prototype, using only features actually
  needed and checked at runtime. Compare on real hardware; wider vectors can
  lose end-to-end. Do not require AVX-512 or globally use `target-cpu=native` in
  portable releases. Check every required bit-manipulation feature too.
- **Scalar/SWAR:** retain a correct non-SIMD path and compare word-at-a-time scans
  with the existing search fallback. SIMD-unavailable must remain a working
  configuration, not an untested `unreachable!` assumption.

Mezura's nibble classification is an experiment source, not a requirement to
compute word masks or adopt its counting model. This counter needs the existing
whitespace definition, not a broader or narrower one chosen for SIMD convenience.
Prefer the currently fused three-mask pass if a separate classification pass
loses. Dispatch outside the inner loop where it actually reduces overhead.

Add a test-only way to exercise each backend independently. Compare masks and
outgoing carry bit-for-bit against a scalar reference for every byte, short tail,
alignment and boundary. Test 15/16, 31/32, 63/64 and 127/128 transitions, empty
input and final partial blocks. Never read beyond an allocation to fill a vector;
zero padding must not create candidates or phantom lines. Large totals must not
wrap a narrow intermediate counter.

**Exit:** measured per-platform dispatch, backend parity, and no instruction-set
or safety regression. A rejected AVX-512 or shuffle experiment is a valid finish.

## P5 — Fewer passes without weakening encoding safety

**Files:** `src/analysis/encoding.rs`, `line_counter.rs`, `masks.rs`, `pipeline.rs`.

Today valid UTF-8 is validated before scanning, while UTF-16 and invalid UTF-8 use
other paths. Establish how much this costs before changing it.

Compare the current `simdutf8` prepass with a byte-oriented common scanner that
can share ASCII/high-bit information with its masks. A high bit only says
“not ASCII”; it does not validate UTF-8. Any fallback must restore the correct
state/count checkpoint and avoid double counting. Do not build `&str` from
unvalidated bytes, and do not insert a separate ASCII prepass that merely replaces
one full pass with two.

Retain current BOM, UTF-16, invalid-sequence replacement, binary-detection and
Unicode behavior unless a separately documented policy change is being tested.
Exercise multibyte characters and delimiters across every new chunk boundary.
Keep `encoding_rs` and `simdutf8` until evidence supports a safe narrower design;
dependency reduction alone is not a parser optimization.

**Exit:** fewer measured byte passes on representative text, exact decoding/count
behavior, and no regression on non-ASCII or malformed input.

## P6 — Pipeline wins that do not require a new architecture

**Files:** `pipeline.rs`, `file_io.rs`, `walk.rs`, `walk/rules.rs`, `stats.rs`,
`langs/detection.rs`, and existing report code only when profiling points there.

Rank these independently; do not land one omnibus “performance refactor.”

1. Avoid a heap-owned candidate vector for the overwhelmingly common single
   language. Compare borrowed/static candidates and compact ambiguous lists.
   Avoid repeated filename/attribute judgment where callers can safely reuse a
   result; do not cache stale directory policy or change override precedence.
2. Measure the sample-copy/detection path, including the 8 KiB combined head/middle
   sample and 64 KiB ambiguous-file probe. Borrow spans or skip unnecessary
   decoding only when the same detection answer and binary policy are retained.
3. Sweep buffered reads versus mapping and thresholds around 64 KiB, 256 KiB and
   1 MiB. Keep buffer reuse, test short/interrupted reads, and measure peak RSS,
   system CPU, faults and mapping contention—not just scanner throughput.
   If larger buffered reads are adopted, cap retained worker scratch so one
   exceptional file does not permanently inflate every worker.
4. Treat memory mapping's stable-file assumption honestly. A recent mtime check
   is not proof that a file cannot change or be truncated. A buffered-read path
   can avoid mapping hazards without promising a coherent snapshot of a changing
   file. Test mutation/error cases separately and document any changed policy.
5. Tune existing lazy worker startup, the wake threshold and work granularity on
   tiny-file and large-file mixes. Compare batched work only where scheduling is
   measurable. Preserve termination, panic/error handling, ignore ordering,
   symlink-loop protection, duplicate suppression and bounded memory. No daemon
   or second thread pool merely to save process startup.
6. Preserve the existing summary-only avoidance of detailed paths. Investigate
   dense language-ID accumulators versus maps, and path/embedded-report cloning,
   only if allocation profiles justify them. Human/JSON output remains correct
   and deterministic; faster counting must not hide slower report construction.

**Exit:** independent, attributable improvements with unchanged selection and
error contracts. No allocator switch without a separate musl/macOS experiment.

## P7 — Approximation is an experiment with a budget

The product may trade a little accuracy for a lot of speed. Do not make exact
optimization depend on achieving perfect parsing of every rare language, but do
not use “approximate” to hide missing files or broken lexical state either.

Start with a benchmark-only policy selector, not another CLI ecosystem. Test
individual simplifications so their benefit/error can be attributed:

- Reduced work for ambiguous extension classification or a smaller content probe,
  while respecting explicit user language/attribute overrides. Report language
  attribution errors separately from line-class errors.
- A simplified rare-syntax path with a reliable fallback at a recognized boundary.
  Ordinary strings, escapes, nested block comments, raw strings and heredocs in
  the primary languages are not casually discarded: a missed closer can poison
  the rest of a file.
- A host-language-only embedded attribution policy as a distinct metric. Omitting
  embedded report rows does **not** automatically make embedded parsing unnecessary
  for correct parent counts. Measure both cost and semantic differences.

Suggested promotion gate: at least 1.25x end-to-end speed on two representative
workloads against the *optimized exact* engine, and no meaningful regression on
the others. For an ordinary fast counting policy, begin with at most 0.1%
misclassified physical lines overall and 0.5% in each adequately sized primary
language slice of a separately annotated corpus. Report small-language cases as
absolute counts too; do not hide them with a minimum-size cutoff. Record per-file
and worst-case errors, not only pooled totals. Any case whose misclassification
spreads beyond the unsupported construct requires a fallback or rejection.

Those accuracy numbers are proposed budgets, not evidence that they are already
achievable. Require line-level comparison on labeled fixtures: similar aggregate
counts can cancel different mistakes. Use the old parser for differential
regressions and manually/spec-checked cases for accuracy; majority voting among
other counters is not ground truth. Maintain a held-out corpus, not just the
files used to tune the heuristic.

A deliberately different language-attribution metric must be labeled as such; it
cannot pass the ordinary accuracy gate by excluding its known differences.
Keep filename inclusion and physical totals exact on the same selected inputs;
never claim a win obtained by skipping difficult source files.

If a variant earns promotion, expose at most one clearly documented fast policy
through the existing CLI schema. Machine-readable output must identify its
counting policy, and any changed field meaning must follow `AGENTS.md`'s schema
versioning rule. Do not silently change default semantics. If the gain is modest
or the error is uncontrolled, reject it and retain the exact engine. Do not carry
a permanent second parser for an unconvincing benchmark result.

## P8 — Verification, integration and closeout

### Tiered checks

Run the smallest relevant checks during experiments; broad corpus downloads and
cross-machine performance runs belong to an explicit qualification step, not an
automatic escalation on every edit. Existing narrow starting points include:

```sh
cargo test --lib analysis::masks::tests::sweeps_match_lines_across_block_edges
cargo test --lib analysis::line_counter::tests::sweeping_matches_classifying_every_line
```

Confirm filters actually execute tests. Extend the current randomized,
all-language sweeping/classifier comparison rather than replacing it. Add
reduced regressions, deterministic generated token streams, state save/restore,
and forced-backend comparison. Cover BOM/encoding cases, empty/final lines,
comment/string markers inside each other, raw delimiter prefixes, escape parity,
C line splices, interpolation holes, heredocs, nested comments, column-one rules,
embedded switches and huge marker-dense lines. Test runtime scaling when adversarial
input doubles, not just the correctness of one small example.

For an accepted chunk, run relevant library/CLI tests, clippy, and targeted
benchmark cases using the pinned toolchain. `cargo bench --bench bench` and
`make bench` are the existing suite entry points; add filtering through the
existing harness rather than inventing an undocumented command. Keep timing
runs separate from concurrent tests/builds and allocation-heavy diagnostics.

At explicit qualification, run the full existing tests/conformance checks and
pinned real-corpus comparisons, then build/verify the supported distribution
artifacts using `make release` and `make verify-release` on suitable hosts.
LineJudge's recorded failures and `benches/baseline.json` are reviewed evidence,
not files to regenerate blindly until the run turns green. An unavailable host
or tool is an unrun check; report it rather than claiming cross-platform parity.

### Combining and shipping

Benchmark the combined winners again: isolated gains are not additive. Revisit
inlining/code size and branch behavior after combination. Evaluate PGO only
against a non-PGO build of the same code and record its training set; keep a
held-out set and do not attribute PGO's gain to a parser rewrite.

Preserve upstream license/copyright notices for adapted code. Integrate the
smallest winning changes, remove losing paths and diagnostic production overhead,
and update user-facing semantics only when behavior actually changes. Keep the
scalar/reference tests and small reproducible fixtures.

Each completed experiment records: baseline/candidate commits, compiler/target/
profile, corpus identities, exact commands, count/state differences, raw evidence
location, time/CPU/allocation/memory/binary-size deltas, and accept/reject rationale.
Do not add a service or database for these records. A compact result under this
plan plus reusable scripts is enough.

The campaign is complete when the useful experiments have been decided, accepted
changes pass their relevant gates, distribution artifacts are qualified on the
claimed platforms, and unresolved limitations are explicit. “No worthwhile win”
is a valid outcome for an individual experiment; an unmeasured rewrite is not.

## Source anchors

Local source links above refer to the reviewed repository checkpoint; recheck
before editing. External implementation references are pinned for reproducibility:

- [Mezura v3.5.0 mask implementation](https://github.com/subamanis/mezura/blob/v3.5.0/mezura-core/src/engine/masks.rs): ISA dispatch, block masks, byte classes and carry-based line accounting.
- [Mezura v3.5.0 parser](https://github.com/subamanis/mezura/blob/v3.5.0/mezura-core/src/engine/file_parser.rs): stateful interpretation of candidate positions. Borrow small mechanisms, not the surrounding product architecture or counting semantics.
- [Local conformance dialect](.linejudge/dialects/codestats/default.toml) and [recorded results](.linejudge/recorded/codestats.toml): the existing regression contract, not a reason to preserve independently demonstrated bugs.
- [Local benchmark driver](scripts/bench.sh), [instruction-cost diagnostic](scripts/costs.py), and [Rustybench harness](benches/bench.rs): extend the existing development loop.
