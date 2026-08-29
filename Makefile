.PHONY: help build run test fmt lint clean doc

help:
	@echo "Jira MCP - Available commands:"
	@echo "  make build     - Build the project (debug)"
	@echo "  make release   - Build optimized release binary"
	@echo "  make run       - Run the application"
	@echo "  make test      - Run tests"
	@echo "  make fmt       - Format code"
	@echo "  make lint      - Run clippy linter"
	@echo "  make check     - Check without building"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make doc       - Generate documentation"
	@echo "  make dev       - Run in debug mode with logging"

build:
	cargo build

release:
	cargo build --release

run: build
	cargo run

test:
	cargo test

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

check:
	cargo check

clean:
	cargo clean

doc:
	cargo doc --no-deps --open

dev:
	RUST_LOG=jira_mcp=debug cargo run

# Development with auto-reload (requires cargo-watch)
watch:
	cargo watch -x run

# Run all checks
all-checks: fmt-check lint test
	@echo "All checks passed!"
