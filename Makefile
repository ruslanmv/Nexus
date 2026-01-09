SHELL := /bin/bash

CARGO := $(shell command -v cargo 2>/dev/null || echo $$HOME/.cargo/bin/cargo)

.PHONY: install build run test fmt clippy clean

install:
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Rust toolchain not found. Installing via rustup..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "Rust installed. Ensure $$HOME/.cargo/bin is on your PATH (or source $$HOME/.cargo/env)."; \
	fi

build:
	$(CARGO) build --release

run:
	$(CARGO) run --bin nexus-demo

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy -- -D warnings

clean:
	$(CARGO) clean
