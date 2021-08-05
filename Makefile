.PHONY: test build

test:
	@${MAKE} -C cao-lang test
	cd build && cargo build && ninja && ctest
	cd py && cargo test
	@${MAKE} -C wasm test
	tox -e py39

update:
	cargo update
	cd wasm && cargo update

build:
	@${MAKE} -C wasm build
