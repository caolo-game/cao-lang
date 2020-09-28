build:
	wasm-pack build --scope caolo-game --dev -d out/pkg

release:
	wasm-pack build --scope caolo-game --release -d out/pkg -- --no-default-features

test:
	cargo check
	wasm-pack test --firefox --headless
	wasm-pack test --chrome --headless
	wasm-pack test --node

pack: release
	wasm-pack pack out/pkg

publish: release
	cd pkg && npm publish --access=public

docs: build
	cd docs && npx typedoc --inputFiles ../out/pkg/cao_lang_wasm.d.ts
