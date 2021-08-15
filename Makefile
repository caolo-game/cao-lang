.PHONY: test build update test-core test-c test-wasm test-py

test-core:
	@${MAKE} -C cao-lang test

test-c:
	cargo xtask test c -- -GNinja

test-py:
	cd py && cargo test
	tox

test-wasm:
	@${MAKE} -C wasm test

test: test-core test-c test-wasm test-py

update:
	cargo update
	cd wasm && cargo update

build:
	@${MAKE} -C wasm build
	python -m build --wheel

