Prefix: LOCC
Next: 5

# Open work
### LOCC-3 · P2 · Qualify x86 scanner mutation coverage

- **Area:** counting tests
- **Where:** src/analysis/masks.rs::BlockScanner and scan_block
- **Problem:** Unproven: ARM mutation runs cannot exercise the x86 AVX2 scanner decisions; scanner tests return early when the required CPU features are absent, so those runs do not establish x86 coverage.
- **Recommendation:** On an x86 host with AVX2, POPCNT, LZCNT and BMI1, exercise the scanner boundary tests and assess the x86-specific mutation survivors; add behavioral coverage where they expose a real gap.
- **Fix with:** test-audit
- **Effort:** M

### LOCC-1 · P3 · Benchmark real CLI journeys and validate completed work

- **Area:** benchmarks
- **Where:** benches/bench.rs and src/analysis.rs::bench::count_as
- **Problem:** The existing generated-tree and fixture workloads discard counts without a correctness oracle and omit CLI startup, filtering, stdin, diff, process memory and I/O; in-memory cases are synthetic. Encoding sample-length mutations in count_as remain unqualified.
- **Recommendation:** Add representative CLI workloads with literal output and completed-work checks, cover encoding sample boundaries in the benchmark helper, and record process-wide CPU, memory and I/O separately from timing.
- **Fix with:** bench-pyramid
- **Effort:** M

### LOCC-2 · P3 · Make benchmark comparisons attributable and paired

- **Area:** benchmarks
- **Where:** benches/baseline.json and tools/bench.py
- **Problem:** The stored baseline is host-specific; selective runs do not perform interleaved revision comparisons or collect profiler, syscall and process-footprint evidence. Busy-host measurements are not absolute performance verdicts.
- **Recommendation:** Qualify paired comparisons across revisions, host readiness and separate CPU, allocation and I/O attribution for the representative workloads.
- **Fix with:** bench-pyramid
- **Effort:** M

### LOCC-4 · P3 · Shebang mutation trial fails every pull request that changes no Rust

- **Area:** ci · mutation
- **Where:** .github/workflows/mutation-trial.yml; Makefile mutants-diff
- **Problem:** The `Shebang mutation` workflow runs on every pull request and `make mutants-diff` exits 1 when the diff yields no mutants (`counts: caught=0, missed=0, unviable=0`, wall time 0.0s), so a workflow- or docs-only pull request is red on all three runners (seen on joshuarli/loc#32, which only changed .github/workflows/lint.yml and added .github/tools.lock.json). The cargo-mutants install also falls back to cargo-binstall on aarch64 because taiki-e/install-action lacks 27.1.0 for aarch64_linux.
- **Recommendation:** Treat an empty mutant set as a pass (or restrict the workflow with `paths:` to the Rust sources it mutates), and install cargo-mutants from a pinned owned or cargo source rather than through install-action's fallback.
- **Fix with:** none
- **Effort:** S
