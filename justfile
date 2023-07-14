test-core:
	@just cao-lang/test

test-c:
	cargo xtask test c -- -GNinja

test-py:
	cd py && cargo test
	tox -p auto

test-wasm:
	just wasm/test

test: test-core test-c test-wasm test-py

update:
	cargo update
	cd wasm && cargo update

build:
	just wasm/build
	python -m build --wheel

