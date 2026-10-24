.PHONY: help build test fmt lint docs docs-links check clean

# The developer commands of PhairPlayRust. Every target below is a thin,
# honest wrapper around a command that can also be typed by hand.
help:
	@echo "PhairPlayRust developer commands:"
	@echo "  make build       - Build every crate of the workspace"
	@echo "  make test        - Run the whole test suite"
	@echo "  make fmt         - Format the Rust sources with cargo fmt"
	@echo "  make lint        - Run cargo clippy with strict warnings"
	@echo "  make docs        - Build the API documentation of the workspace"
	@echo "  make docs-links  - Check that every internal documentation link resolves"
	@echo "  make check       - Run every check that a change must pass"
	@echo "  make clean       - Remove the build artefacts"

build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

docs:
	cargo doc --workspace --no-deps

docs-links:
	bash scripts/check-docs-links.sh

check: fmt-check lint test docs-links

# The format check is separate from `fmt` on purpose: `make fmt` rewrites the
# sources, `make check` must only report.
.PHONY: fmt-check
fmt-check:
	cargo fmt --all --check

clean:
	cargo clean
