# pico-ecs Benchmark Results

Run: `cargo bench --bench ecs_benchmarks` on Intel Core i7-9750H 2.60GHz, Linux.
Calls per frame assumes a 120 FPS frame budget of 8,333,333 ns (1/120 s).

| Operation | Store Size | Time | Calls per frame |
|---|---|---|---|
| add/bulk | 1000 | 56.098 µs | **148x** |
| add/bulk | 10000 | 534.03 µs | **15x** |
| get_by_id | 1000 | 18.537 ns | **449,566x** |
| get_by_id | 10000 | 19.746 ns | **422,052x** |
| each/iterate | 1000 | 331.49 ns | **25,139x** |
| each/iterate | 10000 | 2.820 µs | **2,954x** |
| remove/batch | 1000 | 3.009 µs | **2,769x** |
| remove/batch | 10000 | 3.274 µs | **2,545x** |
| all/collect | 1000 | 2.810 µs | **2,965x** |
| all/collect | 10000 | 30.410 µs | **274x** |
| all_slice/view | 1000 | 27.409 ns | **304,006x** |
| all_slice/view | 10000 | 27.681 ns | **301,043x** |
| first | 1000 | 28.615 ns | **291,212x** |
| first | 10000 | 29.083 ns | **286,538x** |
| descendants | chain of 100 | 1.220 µs | **6,833x** |
| remove_with_children | 100 with children | 17.261 µs | **482x** |

**Bottlenecks:** `add/bulk` at 10k (534 µs, 15x/frame) and `all/collect` at 10k (30.4 µs, 274x/frame) are the slowest paths. `get_by_id`, `all_slice`, and `first` are effectively free — all under 30 ns/call, enabling 280k–450k lookups per frame with zero measurable budget impact.
