# Counting scanner cache contract

`BlockScanner::load` caches masks by block offset and searched byte slice. A later sweep in the same block can search for different markers after the classifier changes state. The regression `sweep_reloads_a_block_when_the_markers_change` checks both the stopping positions and accumulated line counts across that transition.

On Darwin arm64 at base `92ffad1965ff9f3662b257922c0b50f4279d83cc`, replacing the loader's `||` with `&&` made the focused test fail: the second sweep returned `(6, true)` instead of `(13, true)`. Restoring `||` passed it. This is an injected fault, not a discovered production defect. The change adds only a test, so it does not change runtime code or require a runtime benchmark.

## Mutation inventory

The bounded component is the complete `BlockScanner::load` function, selected by its full function name. Cargo-mutants 27.1.0 generated this raw list, with SHA-256 `80e48eba248f9cdd704481f52a24323f832e5ca89ce11ff725bb4caa54a93c97` including newlines:

```text
src/analysis/masks.rs:63:3: replace BlockScanner<'a>::load with ()
src/analysis/masks.rs:63:29: replace || with && in BlockScanner<'a>::load
src/analysis/masks.rs:63:20: replace != with == in BlockScanner<'a>::load
src/analysis/masks.rs:63:32: delete ! in BlockScanner<'a>::load
```

Before and after the test, the three operator mutants were caught. The body replacement was unviable because the crate denies warnings for unused parameters and dead code. It was not tested at runtime. There were no missed mutants or timeouts. The new regression also failed in the final `||` to `&&` mutant run. The result files had these SHA-256 hashes in both runs:

| Result file | SHA-256 |
| --- | --- |
| `caught.txt` | `b51affacc30e9f6452c749bc975de147e1993c6280f5a0ece741718a8d5f853d` |
| `unviable.txt` | `7a6d86401307eac0ca6286df65d8b7bb6cc4120f13d4382de2996d517f177d22` |
| `missed.txt` and `timeout.txt` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

To reproduce, put cargo-mutants 27.1.0 on PATH, acquire the shared mutation lock, and run from the worktree root. The run mutates files in place, so do not build in the same tree concurrently.

```sh
mkdir /Users/josh/d/stuff/.work/tmp/mutation.lock
mkdir -p .work/tmp
cargo mutants --no-config --file src/analysis/masks.rs --re "BlockScanner<'a>::load" --list > .work/tmp/load-list.txt
shasum -a 256 .work/tmp/load-list.txt
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 RUSTC_WRAPPER=kache TMPDIR="$PWD/.work/tmp" cpu-guard --cores 1 -- cargo mutants --no-config --file src/analysis/masks.rs --re "BlockScanner<'a>::load" --in-place --timeout 60 --build-timeout 180 --output .work/tmp/load-after --no-times -- --lib
shasum -a 256 .work/tmp/load-after/mutants.out/{caught,missed,timeout,unviable}.txt
rmdir /Users/josh/d/stuff/.work/tmp/mutation.lock
```

The inventory and results cover the cache loader, not the rest of `masks.rs` or other counting components. The scanner test returns early on machines without the required scanner features.
