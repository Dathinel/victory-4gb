# Reproducing the #419 cache integration

Tested base: fxceless/victory-fork main 006c353 and PR #419 head 0fa4959. In an isolated clean checkout at that main commit, merge the PR head without committing. Only crates/victory-gpu/src/shader_cache.rs conflicts. For this exact pair, select the PR version of that file with git checkout --theirs -- crates/victory-gpu/src/shader_cache.rs, then git apply prefilter-cache.patch and stage the resolved file. Other paths come from the ordinary merge.

The resolution uses main's translator fingerprint and preserves the PR's stage-aware dispatch contract. The disk-cache test uses a synthetic compute shader with the Prefilter contract and counts translations across hits, damage and rebuild. An additional test checks that the actual translator fingerprint contains the value of VICTORY_SPLIT_PREFILTER.

Run cargo test --locked -p victory-gpu --lib shader_cache::tests first with VICTORY_SPLIT_PREFILTER unset and again with it set to 1. Four passed in each process. Three additional CPU tests passed with filter dispatch_contract_tests. These checks do not measure rendered output or GPU performance.
