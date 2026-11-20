Prefix: LOCC
Next: 3

# Open work
### LOCC-1 · P3 · Benchmark real CLI journeys and validate completed work

- **Area:** benchmarks
- **Where:** benches/bench.rs and benchmark entrypoint
- **Problem:** The existing generated-tree and fixture workloads discard counts without a correctness oracle and omit CLI startup, filtering, stdin, diff, process memory and I/O; in-memory cases are synthetic.
- **Recommendation:** Add representative CLI workloads with literal output and completed-work checks and record process-wide CPU, memory and I/O separately from timing.
- **Fix with:** bench-pyramid
- **Effort:** M

### LOCC-2 · P3 · Make benchmark comparisons attributable and paired

- **Area:** benchmarks
- **Where:** benches/baseline.json and tools/bench.py
- **Problem:** The stored baseline is host-specific; selective runs do not perform interleaved revision comparisons or collect profiler, syscall and process-footprint evidence. Busy-host measurements are not absolute performance verdicts.
- **Recommendation:** Qualify paired comparisons across revisions, host readiness and separate CPU, allocation and I/O attribution for the representative workloads.
- **Fix with:** bench-pyramid
- **Effort:** M
