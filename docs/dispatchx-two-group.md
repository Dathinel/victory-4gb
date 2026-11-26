# DispatchX partial groups on RTX 3050 Laptop

Tested [fxceless/victory-fork PR 646](https://github.com/fxceless/victory-fork/pull/646) at 36a2257b, integrated onto local revision 504db0e. Two conflicting files required retaining both sets of exports and both sets of tests. The PR's dispatch dimension conversion test passed. Its single-group execution test ran limits 45, 1 and 0 on hardware wave32. The adapter skipped the native wave64 pipeline.

The additional synthetic test launches two groups of 64. Each group writes to a separate 512-byte region selected by its group ID. Each lane stores its guest-wave active count and bitwise OR. For each requested limit from 65 through 128 the first group must remain full, while the second contains 1 through 64 active lanes. The final control uses limit 0, which disables the cap. Inactive output words must remain zero.

Result: 65 dispatches, 256 words checked per dispatch, all 16,640 correct on the RTX 3050 Laptop. This tests an X-axis tail using the shader and hardware harness. It does not exercise the game's builder, device memory residency, visual output, or frame rate.

Reproduction on Windows: check out PR head 36a2257b, save the adjacent test patch, apply it with Git and run:

```powershell
cargo test --locked -p victory-shader --lib two_groups_with_a_partial_second_wave -- --nocapture
```

The device must create the wave32 pipeline and the test must print that all 65 limits ran on hardware. The patch was checked for applicability on the PR head. The recorded hardware run used the integration described above. AI-assisted.
