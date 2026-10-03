.PHONY: build test lint clean check run

build:
	node scripts/run-in-safe-cwd.mjs -- cargo build --manifest-path src-tauri/Cargo.toml --release

check:
	node scripts/run-in-safe-cwd.mjs -- cargo check --manifest-path src-tauri/Cargo.toml

test:
	node scripts/run-in-safe-cwd.mjs -- cargo test --manifest-path src-tauri/Cargo.toml

lint:
	node scripts/run-in-safe-cwd.mjs -- cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

run:
	node scripts/run-in-safe-cwd.mjs -- cargo run --manifest-path src-tauri/Cargo.toml

clean:
	node scripts/run-in-safe-cwd.mjs -- cargo clean --manifest-path src-tauri/Cargo.toml
