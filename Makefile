test:
	@${MAKE} -C cao-lang test
	cd cli && cargo test
	@${MAKE} -C wasm test

build:
	cd cli && cargo build --release
	@${MAKE} -C wasm build
