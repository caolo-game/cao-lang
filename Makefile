.PHONY: test build

test:
	@${MAKE} -C cao-lang test
	cd cli && cargo test
	@${MAKE} -C wasm test
	cd py && cargo test
	tox -e py39

build:
	cd cli && cargo build --release
	@${MAKE} -C wasm build
