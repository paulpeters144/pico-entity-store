import platform
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent.resolve()

sys.path.insert(0, str(SCRIPT_DIR))
import benchmark_table

CRITERION_DIR = PROJECT_ROOT / "target" / "criterion"
LOG_DIR = PROJECT_ROOT / "benches" / "log"

DOCKER_IMAGE = "rust:1.85-slim-bookworm"
DOCKER_ALPINE = "alpine:3.21"

CPU = "0"
MEMORY = "2g"
CPU_COUNT = "1"


def _docker_rmrf(target: str) -> None:
    subprocess.run(
        ["docker", "run", "--rm", "-v", f"{PROJECT_ROOT}:/app", "-w", "/app", DOCKER_ALPINE, "rm", "-rf", target],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )


def _get_governor() -> str | None:
    return _read_sysfs("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")


def _set_governor(governor: str) -> None:
    subprocess.run(
        [
            "docker", "run", "--rm", "--privileged",
            "-v", "/sys/devices/system/cpu:/sys/devices/system/cpu",
            DOCKER_ALPINE,
            "sh", "-c",
            f"echo {governor} | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1",
        ],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )


def _read_sysfs(path: str) -> str | None:
    try:
        return Path(path).read_text().strip()
    except (OSError, FileNotFoundError):
        return None


def _cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except (OSError, FileNotFoundError):
        pass
    return "unknown"


def _mem_total() -> str:
    try:
        for line in Path("/proc/meminfo").read_text().splitlines():
            if line.startswith("MemTotal:"):
                kb = int(re.findall(r"\d+", line)[0])
                return f"{kb / 1024**2:.1f} GB"
    except (OSError, FileNotFoundError):
        pass
    return "unknown"


def collect_system_info() -> dict[str, str]:
    returning = {
        "timestamp": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        "os": platform.platform(),
        "cpu_model": _cpu_model(),
        "cpu_cores": f"{CPU_COUNT} (pinned to core {CPU})",
        "memory": f"{MEMORY} (container) / {_mem_total()} (system)",
        "image": DOCKER_IMAGE,
    }

    clocksource = _read_sysfs("/sys/devices/system/clocksource/clocksource0/current_clocksource")
    if clocksource:
        returning["clocksource"] = clocksource

    governor = _read_sysfs("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
    if governor:
        returning["governor"] = governor

    return returning


def render_header(info: dict[str, str]) -> str:
    label_width = max(len(k.replace("_", " ").title()) for k in info) + 1

    lines = [
        "=" * 72,
        "Benchmark Environment",
        "=" * 72,
    ]
    for key, value in info.items():
        label = key.replace("_", " ").title()
        lines.append(f"{label + ':':<{label_width}} {value}")
    lines.append("=" * 72)

    return "\n".join(lines)


def main() -> None:
    if shutil.which("docker") is None:
        print("docker not found in PATH. bench-log requires Docker for isolated benchmarks.", file=sys.stderr)
        sys.exit(1)

    if CRITERION_DIR.exists():
        _docker_rmrf("target/criterion")

    prev_governor = _get_governor() or "powersave"
    _set_governor("performance")
    try:
        result = subprocess.run(
            [
                "docker", "run", "--rm",
                "--cpuset-cpus=0",
                "--cpus=1",
                "--memory=2g",
                "--cap-add=SYS_NICE",
                "-v", f"{PROJECT_ROOT}:/app",
                "-w", "/app",
                DOCKER_IMAGE,
                "nice", "-n", "-20", "cargo", "bench", "--bench", "ecs_benchmarks",
            ],
            check=False,
        )
        if result.returncode != 0:
            _docker_rmrf("target/criterion")
            sys.exit(result.returncode)

        results = benchmark_table.collect_benchmarks()
        if not results:
            print("No benchmark results found in target/criterion.", file=sys.stderr)
            _docker_rmrf("target/criterion")
            sys.exit(1)

        header = render_header(collect_system_info())
        table = benchmark_table.render_table(results)
        full_log = header + "\n" + table

        LOG_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y-%m-%d_%H%M")
        log_path = LOG_DIR / f"{timestamp}_benchmark.log"
        log_path.write_text(full_log)

        print(full_log)

        if CRITERION_DIR.exists():
            _docker_rmrf("target/criterion")
    finally:
        _set_governor(prev_governor)


if __name__ == "__main__":
    main()
