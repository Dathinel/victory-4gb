# Reproducing the #419 cache integration

Tested base: fxceless/victory-fork main 006c353 and PR #419 head 0fa4959. In an isolated clean checkout at that main commit, merge the PR head without committing. Only crates/victory-gpu/src/shader_cache.rs conflicts. For this exact pair, select the PR version of that file with git checkout --theirs -- crates/victory-gpu/src/shader_cache.rs, then git apply prefilter-cache.patch and stage the resolved file. Other paths come from the ordinary merge.

The resolution uses main's translator fingerprint and preserves the PR's stage-aware dispatch contract. The disk-cache test uses a synthetic compute shader with the Prefilter contract and counts translations across hits, damage and rebuild. An additional test checks that the actual translator fingerprint contains the value of VICTORY_SPLIT_PREFILTER.

Run cargo test --locked -p victory-gpu --lib shader_cache::tests first with VICTORY_SPLIT_PREFILTER unset and again with it set to 1. Four passed in each process. Three additional CPU tests passed with filter dispatch_contract_tests. These checks do not measure rendered output or GPU performance.

## Building the shader test suite

After resolving the cache as above, apply prefilter-test-build.patch. The scalar-load test on main constructs an unrestricted synthetic compute shader. Its initializer requires the new dispatch_contract field; without it the shader test binary fails with E0063. The one-line patch affects only that test.

All seven synthetic bitcode::group_sync::tests passed, including the prefilter gate and barrier-order checks. The scalar_buffer_loads_read_the_same_dwords_one_call_or_four test ran on WARP in separate processes with VICTORY_SHADER_OPT unset and set to 64. All six offset pairs per run produced the expected values. Capture-dependent translation and hardware tile tests were not run.

## CLI smoke test

The diagnostics build at this integration was exercised through the public CLI. The fixture below is a generated 64-thread compute program with a single end-program instruction, no resources, no capture and no game data. It checks the ordinary translation path when the opt-in prefilter is enabled. It does not match the prefilter itself.

In PowerShell, from the checkout, use an isolated target directory as usual:

```powershell
cargo build --locked -p victory --features diagnostics
$fixture = Join-Path $env:TEMP ([guid]::NewGuid().ToString() + '.dxbc')
[IO.File]::WriteAllBytes($fixture, [Convert]::FromBase64String('RFhCQwAAAAAAAAAAAAAAAAAAAAABAAAArQAAAAEAAAAkAAAARFhJTIEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABYREFUAQAAACQAAAANAAAAMAAAAFEAAABYQkNIAwAAADkAAAAAAAAABAEAAAAAAAB4nHMLDvZlZGRg+MIAAWxQ+v2+tfdIwVBtcIBLPANKf4DSLOgKyASkmOPAwMDACMUMDI37AU3RMig='))
$env:VICTORY_HOST_MAX_WAVE = '32'
$env:VICTORY_SPLIT_PREFILTER = '1'
cargo run --locked -p victory --features diagnostics -- translate $fixture --validate --list --jobs 1
```

With the switch unset and again at 1, the Windows SDK 10.0.26100 validator accepted one of one compute programs (#b3c32985f3206977). Pipeline creation and dispatch were not requested.
