test:
	cd cao-lang && cargo test --benches
	cd cli && cargo test
	${MAKE} -C wasm test

build:
	cd cli && cargo build --release
	${MAKE} -C wasm build
