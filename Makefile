# -------------------------------------jenkins-------------------------------------
.PHONY: kill
kill:
	bash scripts/kill.sh

.PHONY: purge
purge:
	bash scripts/purge.sh


# -------------------------------------development-------------------------------------
.PHONY: install
install:
	cargo install --force --path .

.PHONY: time
time:
	cargo +nightly build -Z timings

.PHONY: clear
clear:
	rm -rf target

.PHONY: echo
echo:
	echo hello

.PHONY: debug
debug:
	cargo build

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --all-targets --features runtime-benchmarks

.PHONY: benchmarks
benchmarks:
	cargo build --bin dico-dev --features runtime-benchmarks

.PHONY: build-dev
build-dev:
	cargo build --bin dico-dev

.PHONY: release
release: clear
	cargo build --release

.PHONY: fix
fix:
	pre-commit run --all-files

.PHONY: dev
dev:
	RUST_LOG=runtime=debug ./target/debug/dico-dev --dev --ws-external

# -------------------------------------cargo-------------------------------------

.PHONY: meta
meta:
	cargo metadata --verbose --format-version 1 --all-features


# -------------------------------------git-------------------------------------
.PHONY: diff
diff:
	git diff --name-only --diff-filter=U
