.PHONY: kill
kill:
	bash scripts/kill.sh

.PHONY: purge
purge:
	bash scripts/purge.sh


.PHONY: install
install:
	cargo install --force --path .

.PHONY: time
time:
	cargo +nightly build -Z timings

.PHONY: clear
clear:
	rm -rf target

.PHONY: debug
debug:
	cargo build

.PHONY: build
build:
	cargo build

.PHONY: build-dev
build-dev:
	cargo build --bin dico-dev --locked

.PHONY: release
release: clear
	cargo build --release

.PHONY: fix
fix:
	pre-commit run --all-files

.PHONY: dev
dev:
	./target/release/dico-dev --dev --ws-external

# -------------------------------------cargo-------------------------------------

.PHONY: meta
meta:
	cargo metadata --verbose --format-version 1 --all-features


# -------------------------------------git-------------------------------------
.PHONY: diff
diff:
	git diff --name-only --diff-filter=U
