# Adjacent short texture views

HLVM04 identified and tested the size-bounded head rule on a 5070 Ti: https://github.com/fxceless/victory-fork/issues/923#issuecomment-7173284806 . This is an independent implementation and a synthetic reproducer for that report. It makes no new visual or performance claim.

On fxceless main 006c353 the second of two adjacent 256-byte textures incorrectly evicts the first slot. Apply short-texture-heads.patch for the test and fix. For the negative control keep the new test and restore just the original Rule::Start match arm. Run cargo test --locked -p victory-gpu --lib small_texture_heads. It fails on the baseline with stale slot [1] instead of []. With the correction, disjoint heads survive in either order, including unequal sizes; a one-byte overlap still evicts. The 20 descriptor::host::tests passed in the integration that includes #419. The descriptor file at the tested base was identical to 006c353.

The ordered lookup uses the incoming texture end as an upper bound and excludes preceding textures that end at or before its start. The four-KiB maximum still applies to both heads. An independent arithmetic check matched direct half-open interval intersection on 1,728 pairs including addresses near the u64 limit.
