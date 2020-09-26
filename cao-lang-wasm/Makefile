build:
	wasm-pack build --scope caolo-game --dev

release:
	wasm-pack build --scope caolo-game --release -- --no-default-features

test:
	cargo check
	wasm-pack test --firefox --headless
	wasm-pack test --chrome --headless

testff:
	cargo check
	wasm-pack test --firefox --headless

pack:
	wasm-pack pack

publish: release
	cd pkg && npm publish --access=public
