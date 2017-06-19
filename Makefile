
fmt:
	rustup run nightly cargo fmt -- --write-mode overwrite

unit:
	cargo test --lib --no-fail-fast

integration:
	cargo test

ignored:
	cargo test -- --ignored

