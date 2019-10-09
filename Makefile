build:
	@cargo build --target wasm32-unknown-unknown --release
lint:
	@cargo fmt