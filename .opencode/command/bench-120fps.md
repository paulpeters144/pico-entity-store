---
description: Run pico-ecs benchmarks and report calls-per-frame at 120 FPS
---

Run `cargo bench` in the repo root. Parse the output lines matching `time:   [...]` to extract the median (middle) time value and unit for each benchmark. Compute how many times each operation can be called within one frame at 120 FPS (1/120 ≈ 8.333 ms = 8,333,333 ns).

Present as a markdown table with columns: Operation, Store Size, Time, Calls per frame. The "Calls per frame" column should be bolded integers with an "x" suffix (e.g. **114x**). For operations that take longer than one frame, show **<1x** in bold.

Group the rows like:
- add/bulk (1000 and 10000)
- get_by_id (1000 and 10000)
- each/iterate (1000 and 10000)
- remove/batch (1000 from 1000, 1000 from 10000)

End the output with a 1-2 sentence key takeaway highlighting bottlenecks and free operations.
