.PHONY: test bench bench-save

test:
	cargo check-all-features
	cargo test --tests --benches
	cargo test --doc

bench:
	cargo bench --bench cao_lang_benches $(benches) -- --baseline master

bench-save:
	cargo bench --bench cao_lang_benches $(benches) -- --save-baseline master
