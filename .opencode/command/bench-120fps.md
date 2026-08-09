---
description: Run pico-ecs benchmarks and report calls-per-frame at 120 FPS
---

## Step 1: Run benchmarks

Run `cargo bench` in the repo root.

## Step 2: Save results to log file

**MANDATORY — do this before presenting results to the user.**

Create the file `benches/log/YYYY-MM-DD_HHMM_benchmark.md` (timestamped with the current date/time, overwriting any existing file with that name). The file must contain the full results table and takeaways described below.

## Step 3: Present results

Parse the benchmark output lines matching `time:   [...]` to extract the median (middle) time value and unit for each benchmark. Compute how many times each operation can be called within one frame at 120 FPS (1/120 ≈ 8.333 ms = 8,333,333 ns).

Present as a markdown table with columns: Operation, Store Size, Time, Calls per frame. The "Calls per frame" column should be bolded integers with an "x" suffix (e.g. **114x**). For operations that take longer than one frame, show **<1x** in bold.

Group the rows like:
- add/bulk (1000 and 10000)
- get_by_id (1000 and 10000)
- each/iterate (1000 and 10000)
- remove/batch (1000 from 1000, 1000 from 10000)

End the output with a 1-2 sentence key takeaway highlighting bottlenecks and free operations.
