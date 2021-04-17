.PHONY: test build

test:
	@${MAKE} -C cao-lang test
	@${MAKE} -C wasm test
	cd py && cargo test
	tox -e py39

build:
	@${MAKE} -C wasm build
