all:
	cat Makefile | grep '^[a-z]' | tr -d ':' | awk '{ print $$1 }'

clean:
	cargo clean

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

