.PHONY: test bench bench-log lint publish

test:
	cargo nextest run

lint:
	cargo clippy --all-targets -- -D warnings
	cargo check --all-targets

bench:
	@rm -rf target/criterion
	cargo bench --bench ecs_benchmarks
	python3 scripts/benchmark_table.py
	@rm -rf target/criterion

bench-log:
	python3 scripts/bench_log.py

publish:
	python3 scripts/publish.py
