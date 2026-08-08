.PHONY: test bench

test:
	cargo nextest run

bench:
	@rm -rf target/criterion
	cargo bench --bench ecs_benchmarks
	@rm -rf target/criterion
