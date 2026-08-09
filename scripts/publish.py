import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent.resolve()
CARGO_TOML = PROJECT_ROOT / "Cargo.toml"

VERSION_LINE_RE = re.compile(r'^(version\s*=\s*)"[^"]+"\s*$')


def _run(cmd: list[str]) -> subprocess.CompletedProcess:
    print(f"$ {' '.join(cmd)}")
    return subprocess.run(cmd, check=False)


def current_version() -> str:
    with CARGO_TOML.open("rb") as f:
        data = tomllib.load(f)
    return data["package"]["version"]


def bump_version(segment: str, version: str) -> str:
    major, minor, patch = (int(part) for part in version.split("."))
    if segment == "major":
        return f"{major + 1}.0.0"
    if segment == "minor":
        return f"{major}.{minor + 1}.0"
    if segment == "patch":
        return f"{major}.{minor}.{patch + 1}"
    raise ValueError(f"unknown bump segment: {segment}")


def write_version(version: str) -> None:
    lines = CARGO_TOML.read_text().splitlines()
    for i, line in enumerate(lines):
        if VERSION_LINE_RE.match(line):
            lines[i] = VERSION_LINE_RE.sub(r'\1"' + version + '"', line)
            break
    else:
        raise RuntimeError(f"could not find version line in {CARGO_TOML}")
    CARGO_TOML.write_text("\n".join(lines) + "\n")


def confirm(version: str) -> bool:
    print(f"\nAbout to publish v{version} to crates.io.")
    answer = input("Proceed? [y/N] ").strip().lower()
    return answer in ("y", "yes")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run pre-publish checks and publish the crate to crates.io.",
    )
    parser.add_argument(
        "--bump",
        choices=["major", "minor", "patch"],
        default="patch",
        help="Bump the version in Cargo.toml before publishing (default: patch).",
    )
    parser.add_argument(
        "-y", "--yes",
        action="store_true",
        help="Skip the confirmation prompt before publishing.",
    )
    parser.add_argument(
        "--skip-checks",
        action="store_true",
        help="Skip test, clippy, and publish --dry-run checks.",
    )
    parser.add_argument(
        "--no-git",
        action="store_true",
        help="Do not commit the version bump or create a git tag.",
    )
    parser.add_argument(
        "--push",
        action="store_true",
        help="Push the release commit and tag to origin after publishing.",
    )
    args = parser.parse_args()

    version = current_version()

    version = bump_version(args.bump, version)
    write_version(version)
    print(f"Bumped Cargo.toml to v{version}")

    if not args.skip_checks:
        for cmd in (
            ["cargo", "test"],
            ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
            ["cargo", "publish", "--dry-run", "--allow-dirty"],
        ):
            result = _run(cmd)
            if result.returncode != 0:
                print(f"check failed with exit code {result.returncode}", file=sys.stderr)
                sys.exit(result.returncode)

    if not (args.yes or confirm(version)):
        print("aborted")
        sys.exit(1)

    result = _run(["cargo", "publish"])
    if result.returncode != 0:
        sys.exit(result.returncode)

    if args.no_git:
        print(f"published v{version}")
        sys.exit(0)

    _run(["git", "add", "Cargo.toml"])
    result = _run(["git", "commit", "-m", f"chore: release v{version}"])
    if result.returncode != 0:
        print("git commit failed", file=sys.stderr)
        sys.exit(result.returncode)
    result = _run(["git", "tag", "-a", f"v{version}", "-m", f"Release v{version}"])
    if result.returncode != 0:
        print("git tag failed", file=sys.stderr)
        sys.exit(result.returncode)

    if args.push:
        _run(["git", "push", "origin", "HEAD"])
        _run(["git", "push", "origin", f"v{version}"])

    print(f"published v{version}")


if __name__ == "__main__":
    main()
