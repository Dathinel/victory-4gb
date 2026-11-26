# Sampling resident mips with the resource minimum LOD

The resident-range fix from [PR #1031](https://github.com/fxceless/victory-fork/pull/1031) and the resource-coordinate fix from [PR #1210](https://github.com/fxceless/victory-fork/pull/1210) touch different versions of the same function. The accompanying resident-mip-clamp.patch applies to #1031 at db88f96. It adapts #1210 to show_resident. Credit to teecil for identifying the clamp error and providing the committed-texture readback probe (c1adcdcb0698a9e661e0504ac67ea3c93e86fc4f).

The fixture encodes different values at each mip. In the unchanged #1031 probe on WARP, the adjusted and original views returned 48 and 64 when both should select mip 3. After the adaptation, the two views both returned 64 on WARP and on an RTX 3050 Laptop (D3D12 debug layer). The entire four-sample output was [32,16,64,64] on both. The separate reserved-texture fixture still returned 32 through the resident view versus 0 through the old view on both adapters. Three ordinary tile tests passed; both normally ignored readback tests ran and passed on both devices. No frame rate, VRAM reduction or character rendering was measured.

To reproduce: apply the patch on db88f96 and cherry-pick teecil's public test commit. Run the ordinary tiles::tests, then cargo test --locked -p victory-gpu --lib tiles::tests -- --ignored --nocapture --test-threads=1 with VICTORY_TILES_TEST_CARD=0 (WARP) and 1 (hardware). Check the named adapter and numeric readbacks to rule out skips. Only the tile suite is verified here; a misquoted local runner accidentally started a wider GPU suite, which exited with 0xc0000005 and was not used for these results.

### A missing mip and a nonzero minimum together

With a synthetic 8x8 committed texture, mark only mips 1 and 3 resident in the descriptor-selection mask. Sample at explicit LOD 0 with point filtering. The unrestricted reference with a minimum of 3 returns 64 (mip 3). This tests the descriptor choice independently of tile backing. On the integrated #1031 + #1210 baseline the adjusted view selected mip 1 alone, below the required minimum.

| Mask | Minimum | Adapter | Baseline adjusted | Candidate adjusted | Reference |
| --- | ---: | --- | ---: | ---: | ---: |
| 1010 | 3 | NVIDIA RTX 3050 Laptop | 0 | 64 | 64 |
| 1010 | 3 | Microsoft WARP | 32 | 64 | 64 |
| 1110 | 3 | Both | 64 | 64 | 64 |
| 1010 | 2.75 | Both | not measured on the original baseline | 64 | 64 |

[Candidate and fixture](https://github.com/Dathinel/victory-fork/commit/ad8db725ebdbf4045647e89b7d19fe40387f2fe6). To reproduce, use that commit with the exact ignored test tiles::tests::committed_texture_keeps_original_resource_lod_clamp. Set VICTORY_TILES_TEST_CARD=1 for hardware or 0 for WARP. For the before arm restore only show_resident from parent 0aec241; keep the identical test. Both executables were built with the same local Rust toolchain. The baseline failed at the integer-minimum assertion, as expected. Claude independently reviewed the candidate source. Linear filtering, other instructions with view-relative LOD, performance and visible game frames were not evaluated in this experiment.
