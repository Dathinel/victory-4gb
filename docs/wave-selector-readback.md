# Checking a cross-half reduction on the RTX 3050 Laptop

Tested integration: a84cec8 (PR #695 selector on the 4a11834 tree with #799). This synthetic 64-thread compute shader is built from the public wave64_tests.rs OR fixture with the SCC branch. Each active lane t contributes bit t/2 (integer division). The expected word is the OR of all lanes below a threshold. The only experimental change between the two RTX runs is the selected container hash. These are numeric dispatch readbacks, not frame or timing measurements.

| Active lanes | Expected | Ordinary local approximation (observed) | Selected wave-wide |
| ---: | ---: | ---: | ---: |
| 1 | 00000001 | 00000001 | 00000001 |
| 10 | 0000001f | 0000001f | 0000001f |
| 31 | 0000ffff | 0000ffff | 0000ffff |
| 32 | 0000ffff | 0000ffff | 0000ffff |
| 33 | 0001ffff | 00010000 | 0001ffff |
| 40 | 000fffff | 000f0000 | 000fffff |
| 63 | ffffffff | ffff0000 | ffffffff |
| 64 | ffffffff | ffff0000 | ffffffff |

The ordinary form produced an incomplete word in four cases; the specific winning half is not guaranteed where both host waves store to the same address. With the exact hash selected all eight matched. Both forms passed DXIL validation offline. WARP refused pipeline creation for both forms on this machine; it supplied no numeric results. These checks do not establish the outputs of the two real water shaders.

Apply wave-selector-readback.patch to a checkout of a84cec8. The fixture below is generated test data. Use PowerShell to run the same test twice:

```powershell
$env:CODEX_OR_FIXTURE = Join-Path $env:TEMP ([guid]::NewGuid().ToString() + '.cdxil')
[IO.File]::WriteAllBytes($env:CODEX_OR_FIXTURE, [Convert]::FromBase64String('RFhCQwAAAAAAAAAAAAAAAAAAAAABAAAARAEAAAEAAAAkAAAARFhJTBgBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABYREFUAQAAACQAAAANAAAAMAAAAOgAAABYQkNIAwAAANAAAAAAAAAAxAEAAAAAAAB4nJWNva5BQRhF1/edieMvIhoegE6pUNLobuU+gFJUSg05JyFR6DyHTj3iIUQnWpVb3ErhimNO4koQK5msnT2TPa12+0sElkJE4iaOdrH75PDAs77rXHX/TXnNj3PJvVfAxX/k3+zEpIGG27jfMXfZc47vjYAXl3FnhMCbWNXkL3AKNT0KyVYTmVwNpsNeeWaLjFfrytgKsglyBQ6qdVl/7yIXOvPI5RSnqxupP2WwFRFVT+s+na00BV9dFsGYs/WVEfT3oEHWnC2Eqws5IVQu'))
$env:VICTORY_WAVE_WHY = '1'
Remove-Item Env:CODEX_OR_WARP -ErrorAction SilentlyContinue
foreach ($selector in 'ffffffffffffffff','7a20f4c11ee2d343') {
    $env:VICTORY_WAVE_WIDE_ONLY = $selector
    cargo test --locked -p victory-shader --lib synthetic_or_selector_readback -- --nocapture
}
```

Fixture SHA-256: 6AD8B9D8761959C075A6EE50E88668D46904C1B7AFC343E8A2A5E8D0D8B181E2. Run in a shell without other shader overrides. On the RTX, the negative control fails at the final mismatch count; the selected run passes. AI-assisted experiment.

## One output per lane

On the same RTX 3050 Laptop integration, a second synthetic fixture writes s7 to byte offset 4*t for each active lane t. Nine thresholds (0, 1, 10, 31, 32, 33, 40, 63, 64) yielded 274 active and 302 inactive slots. The ordinary form returned incorrect values in all 200 active slots belonging to thresholds above 32. At threshold 33, lanes 0 and 32 returned 0000ffff and 00010000; the selective wave-wide form gave 0001ffff to both. The selected form matched the oracle in every active slot. Inactive slots read zero in both arms. The shader bytes and D3D12 adapter were identical between arms; the CLI independently reported the fixture hash below. The adapter reported a wave size range of 32 to 32. Local WARP reported 4 to 4 and refused the requested 32-lane pipeline.

Use a clean checkout of a84cec8, apply wave-selector-lane-readback.patch, and run the commands below. These numbers apply to a synthetic OR, not the water passes. An independent RTX 3070 report found near-white window rendering when both water hashes were selected; the same scene stayed clear without those hashes ([controlled comparison](https://github.com/fxceless/victory-fork/pull/823#issuecomment-7162022167)).

```powershell
$env:CODEX_OR_FIXTURE = Join-Path $env:TEMP ([guid]::NewGuid().ToString() + '.cdxil')
[IO.File]::WriteAllBytes($env:CODEX_OR_FIXTURE, [Convert]::FromBase64String('RFhCQwAAAAAAAAAAAAAAAAAAAAABAAAASwEAAAEAAAAkAAAARFhJTB8BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABYREFUAQAAACQAAAANAAAAMAAAAO8AAABYQkNIAwAAANcAAAAAAAAAyAEAAAAAAAB4nJWNsWoCQRRFz3sz7GaNiFj5Aaaz3MLSNHZW+gGWwcrSJrIbDKSwy3fYW434EWIntqmSwspCg+ssGMGIB4ZzuTPcaXU6bRGYCRnBSXy76fqewwXX+hfvuv9vwv/8eFf9ewV8/EP5xk5OAWj6jfMde5aNd35vBUxe5p0VEvPuVB+2wC7VwiilWA8eSzF8vPZrE1dlPF88jZ0gy6RU4Uu1IYvuOnOl95m5FrE7uhkdlOFKRFSNNkJ6K3kWQvVZBGv37g0bhxqMKA82ajQp2r2DdP4Lk0FVDA=='))
$env:VICTORY_WAVE_WHY = '1'
Remove-Item Env:CODEX_OR_WARP -ErrorAction SilentlyContinue
foreach ($selector in 'ffffffffffffffff','9d5192aaa9e752d2') {
    $env:VICTORY_WAVE_WIDE_ONLY = $selector
    cargo test --locked -p victory-shader --lib emit::wave64_tests::synthetic_or_per_lane_readback -- --exact --test-threads=1 --nocapture
}
```
