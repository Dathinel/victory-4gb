# DispatchX thread-count diagnostics

Optional patch for fxceless/victory-fork PR #646 at 36a2257b. Apply dispatchx-trace.patch to that checkout with git apply. In a diagnostics-enabled build, set VICTORY_LOG=info,victory_gpu::dispatch_x=debug. One debug event records the pipeline name, raw flags, thread dimensions, native group size, dispatched groups and partial limits after a thread-count dispatch is recorded.

A CPU-only synthetic probe exercised 65 threads (flags 0x8020), 128 threads (0x20), ordinary group mode (0x8000) and zero threads. With group size [64,1,1], debug produced two events: 65 and 128 both requested two groups, with partial limits [65,0,0] and [0,0,0] respectively. The same probe at info produced zero dispatch events. Cargo check passed with and without diagnostics on the local integration.

This records command submission to the list. GPU completion, timing and rendered pixels require separate measurements.
