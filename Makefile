all: check unit integration

check:
	cargo $@

build:
	cargo $@

clean:
	cargo $@

fmt:
	rustup run nightly cargo fmt -- --write-mode overwrite

clippy:
	rustup run nightly cargo clippy

tests: integration ignored

unit:
	cargo test --lib --no-fail-fast

integration:
	cargo test

ignored:
	cargo test -- --ignored

.PHONY: tests

