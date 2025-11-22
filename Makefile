all: test

test:
	cargo test -p cwnote

test-verbose:
	cargo test -p cwnote -- --nocapture

linting:
	cargo clippy

clean:
	cargo clean

test: test

.PHONY: all test clean