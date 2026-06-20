# List available recipes
default:
    @just --list

# Build the library and examples
build:
    cargo build --all-targets

# Format code and lint with clippy, treating warnings as errors
lint:
    cargo fmt
    cargo clippy --all-targets -- -D warnings

# Run the test suite
test:
    cargo test

# Run the interactive demo REPL
demo:
    cargo run --example demo
