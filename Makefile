run:
	SKIP_WASM_BUILD= cargo run -- --dev --execution=native -lruntime=debug

toolchain:
	./scripts/init.sh