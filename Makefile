
fmt:
	rustup run nightly cargo fmt -- --write-mode overwrite

unit:
	cargo test --lib

integration:
	cargo test

ignored:
	cargo test -- --ignored

