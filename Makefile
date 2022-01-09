help:
	@printf "\rmake %-30s %-20s\n" benchmark "run benchmark"
	@printf "\rmake %-30s %-20s\n" benchmark_report "open benchmark report"
	@printf "\rmake %-30s %-20s\n" clean_benchmark_report "delete benchmark report"
	@printf "\rmake %-30s %-20s\n" clean "remove temporary files"

benchmark:
	cargo criterion

benchmark_report:
	open ./target/criterion/reports/index.html

clean_benchmark_report:
	rm -rf ./target/criterion

clean: clean_benchmark_report