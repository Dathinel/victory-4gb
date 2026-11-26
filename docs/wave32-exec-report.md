# Wave32 regression on the RTX 3050 Laptop

This experiment tests whether a guest wave64 that skips a branch can accidentally write output when another guest wave in its thread group enters the branch. The fixture is synthetic. It uses a 128-thread group, a restored EXEC mask, an existing public wave-OR instruction sequence, and two 128-word output regions. One store occurs inside the branch. A second store at the join observes the outgoing EXEC mask.

Base: [fxceless/victory-fork PR 1124, commit 887d053](https://github.com/fxceless/victory-fork/commit/887d053c81d1df8a68a1888024ac47517169aa7c). Executor: RTX 3050 Laptop, hardware wave32, Windows, VICTORY_WAVE_WIDE=1. Each run created the pipeline, dispatched thresholds 0 through 128 inclusive and 200, and checked every output word. Counts below are mismatches among 16,640 words per output region.

| Translator | Within branch | After join |
| --- | ---: | ---: |
| PR head | 4,096 | 4,096 |
| Experimental per-wave scalar selection and branch store guard | 0 | 4,096 |
| Also select the skipped wave's incoming mask at the join | 0 | 0 |

To reproduce, use a checkout at the base above. Save the two patches beside the checkout. On Windows PowerShell:

```powershell
git apply ../wave32-exec-regression.patch
$env:VICTORY_WAVE_WIDE = "1"
$env:VICTORY_WAVE_WHY = "1"
cargo test --locked -p victory-shader --lib multi_wave_stores_respect_exec_after_the_join -- --nocapture
git apply ../wave32-exec-candidate.patch
cargo test --locked -p victory-shader --lib multi_wave_stores_respect_exec_after_the_join -- --nocapture
```

The first test should fail on the numeric assertion with [4096, 4096]. The second should pass. Both must reach the wave-wide retry and dispatch on hardware. The experimental fix handles a direct, single-block branch body. Loops, other side effects, rendered image quality and frame rate remain unverified. The patches are published for reproduction and review. AI-assisted development and review.
