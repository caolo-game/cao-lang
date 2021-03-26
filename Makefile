.PHONY: test build

test:
	@${MAKE} -C cao-lang test
	# cli
	cd cli && cargo test
	# wasm
	@${MAKE} -C wasm test
	# python interface
	cd py && cargo test
	tox -e py39 # run the python tests on Python 3.9 only, by default

build:
	cd cli && cargo build --release
	@${MAKE} -C wasm build
