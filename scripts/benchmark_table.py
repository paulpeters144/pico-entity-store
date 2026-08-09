import json
import sys
from pathlib import Path

FRAME_BUDGET_NS = 8_333_333  # 1/120 s

CRITERION_DIR = Path("target/criterion")


def format_duration(ns: float) -> str:
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    elif ns >= 1_000:
        return f"{ns / 1_000:.2f} µs"
    else:
        return f"{ns:.1f} ns"


def format_calls(calls: float) -> str:
    if calls >= 1_000_000:
        return f"{calls / 1_000_000:.2f}M"
    elif calls >= 1_000:
        if calls >= 100_000:
            return f"{calls / 1000:.0f}k"
        return f"{calls / 1000:.1f}k"
    else:
        return f"{calls:.0f}"


def collect_benchmarks() -> list[tuple[str, str, float]]:
    results = []

    for estimates_path in sorted(CRITERION_DIR.rglob("estimates.json")):
        rel = estimates_path.relative_to(CRITERION_DIR)
        parts = rel.parts  # e.g. ("add", "bulk", "1000", "new", "estimates.json")

        data = json.loads(estimates_path.read_text())
        median_ns = data["median"]["point_estimate"]
        calls = FRAME_BUDGET_NS / median_ns

        if parts[0] == "report":
            continue

        if len(parts) == 5:
            # parameterized: group/name/value/new/estimates.json
            bench = "/".join(parts[:2])
            name = parts[2]
        elif len(parts) == 4:
            # unparameterized: group/name/new/estimates.json
            bench = "/".join(parts[:2])
            name = ""
        else:
            continue

        results.append((bench, name, median_ns, calls))

    return results


def render_table(results: list[tuple[str, str, float, float]]) -> str:
    header_bench = "Bench"
    header_name = "Name"
    header_time = "Time (median)"
    header_calls = "Calls / Frame"

    bench_width = max(max(len(b) for b, _, _, _ in results), len(header_bench))
    name_width = max(max(len(n) for _, n, _, _ in results), len(header_name))
    time_width = max(len(header_time), 14)
    calls_width = max(len(header_calls), 14)

    separator = (
        f"| {'-' * bench_width} | {'-' * name_width} | {'-' * time_width} | {'-' * calls_width} |"
    )
    header = f"| {header_bench:<{bench_width}} | {header_name:<{name_width}} | {header_time:>{time_width}} | {header_calls:>{calls_width}} |"

    lines = ["", header, separator]

    for bench, name, ns, calls in results:
        time_str = format_duration(ns)
        calls_str = format_calls(calls)
        lines.append(
            f"| {bench:<{bench_width}} | {name:<{name_width}} | {time_str:>{time_width}} | {calls_str:>{calls_width}} |"
        )

    lines.append("")
    return "\n".join(lines)


def print_table(results: list[tuple[str, str, float, float]]) -> None:
    print(render_table(results))


def main():
    if not CRITERION_DIR.exists():
        print("No criterion output found. Run `cargo bench --bench ecs_benchmarks` first.", file=sys.stderr)
        sys.exit(1)

    results = collect_benchmarks()
    if not results:
        print("No benchmark results found in target/criterion.", file=sys.stderr)
        sys.exit(1)

    print_table(results)


if __name__ == "__main__":
    main()
