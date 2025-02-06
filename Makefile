# Define the binary name
BINARY_NAME := semtag

# Default target
all: build

# Build the project in release mode
build:
	cargo build --release

# Build the project in debug mode
debug:
	cargo build

# Run the project
run:
	cargo run

# Run the project in release mode
run-release:
	cargo run --release

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt --all

# Run Clippy (linter)
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Remove build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --no-deps --open

# Check for outdated dependencies
update:
	cargo update

# Install dependencies
deps:
	cargo fetch

.PHONY: all build debug run run-release test fmt lint clean doc update deps
