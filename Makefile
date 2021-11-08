benchmark:
	cargo criterion

benchmark_report:
	open ./target/criterion/reports/index.html
