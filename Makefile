.PHONY: test build

test:
	@${MAKE} -C cao-lang test
	@${MAKE} -C wasm test
	cd c && cargo build && clang test.c -l ../target/debug/cao_lang_c
	cd py && cargo test
	tox -e py39

build:
	@${MAKE} -C wasm build
