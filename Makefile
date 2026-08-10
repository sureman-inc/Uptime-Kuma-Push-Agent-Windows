# Makefile for Uptime Kuma Push Agent (Rust)

.PHONY: all build release run clean test fmt clippy help

all: build

## build: Build the project in debug mode
build:
	cargo build

## release: Build the project in release mode (optimized)
release:
	cargo build --release

## run: Run the application in debug mode
run:
	cargo run

## clean: Clean build artifacts
clean:
	cargo clean

## test: Run unit and integration tests
test:
	cargo test

## fmt: Format Rust source code using rustfmt
fmt:
	cargo fmt

## clippy: Run clippy linter for static analysis
clippy:
	cargo clippy

## help: Show this help message
help:
	@echo "Available commands in this Makefile:"
	@echo "  make build    - Build the project in debug mode"
	@echo "  make release  - Build the project in release mode (optimized)"
	@echo "  make run      - Run the application in debug mode"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make test     - Run unit and integration tests"
	@echo "  make fmt      - Format Rust source code"
	@echo "  make clippy   - Run clippy linter"
	@echo "  make help     - Show this help message"
