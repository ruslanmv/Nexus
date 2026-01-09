SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

# Project Configuration
PROJECT_NAME := nexus
BINARY_NAME := nexus-runtime
VERSION := 0.2.0
BUILD_DIR := target/release
INSTALL_DIR := $(HOME)/.local/bin
CONFIG_DIR := $(HOME)/.config/nexus
DATA_DIR := $(HOME)/.local/share/nexus

# Build Configuration
CARGO := $(shell command -v cargo 2>/dev/null || echo $$HOME/.cargo/bin/cargo)
CARGO_FLAGS := --release
RUST_LOG ?= info

# Colors for output
NO_COLOR=\033[0m
INFO_COLOR=\033[0;36m
SUCCESS_COLOR=\033[0;32m
ERROR_COLOR=\033[0;31m

.PHONY: all help deps install build run test test-all bench fmt clippy clean distclean setup-dirs setup-config verify doctor deploy uninstall

# Default target
all: build

## help: Show this help message
help:
	@echo "$(INFO_COLOR)Project Nexus - Production Build System$(NO_COLOR)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Core Targets:"
	@echo "  install       - Build and install Nexus binary to $(INSTALL_DIR)"
	@echo "  build         - Build release binary"
	@echo "  run           - Run Nexus with default configuration"
	@echo "  test          - Run unit and integration tests"
	@echo "  clean         - Remove build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  deps          - Install Rust toolchain and dependencies"
	@echo "  fmt           - Format code"
	@echo "  clippy        - Run linter"
	@echo "  test-all      - Run all tests including ignored ones"
	@echo "  bench         - Run benchmarks"
	@echo "  doctor        - Check development environment"
	@echo ""
	@echo "Deployment:"
	@echo "  setup-dirs    - Create required directories"
	@echo "  setup-config  - Create default configuration files"
	@echo "  verify        - Verify installation"
	@echo "  deploy        - Full production deployment"
	@echo "  uninstall     - Remove installed files"
	@echo ""
	@echo "Environment:"
	@echo "  RUST_LOG=$(RUST_LOG)"
	@echo "  INSTALL_DIR=$(INSTALL_DIR)"

## deps: Install Rust toolchain if not present
deps:
	@echo "$(INFO_COLOR)Checking Rust toolchain...$(NO_COLOR)"
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "$(INFO_COLOR)Rust toolchain not found. Installing via rustup...$(NO_COLOR)"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "$(SUCCESS_COLOR)Rust installed. Please run: source $$HOME/.cargo/env$(NO_COLOR)"; \
	else \
		echo "$(SUCCESS_COLOR)Rust toolchain found: $$(rustc --version)$(NO_COLOR)"; \
	fi
	@echo "$(INFO_COLOR)Checking cargo components...$(NO_COLOR)"
	@$(CARGO) --version || (echo "$(ERROR_COLOR)Cargo not found in PATH$(NO_COLOR)" && exit 1)
	@echo "$(SUCCESS_COLOR)Dependencies OK$(NO_COLOR)"

## setup-dirs: Create required directories for installation
setup-dirs:
	@echo "$(INFO_COLOR)Creating directory structure...$(NO_COLOR)"
	@mkdir -p $(INSTALL_DIR)
	@mkdir -p $(CONFIG_DIR)
	@mkdir -p $(DATA_DIR)/logs
	@mkdir -p $(DATA_DIR)/metrics
	@echo "$(SUCCESS_COLOR)Directories created:$(NO_COLOR)"
	@echo "  Binary: $(INSTALL_DIR)"
	@echo "  Config: $(CONFIG_DIR)"
	@echo "  Data:   $(DATA_DIR)"

## setup-config: Create default configuration file
setup-config: setup-dirs
	@echo "$(INFO_COLOR)Setting up configuration...$(NO_COLOR)"
	@if [ ! -f $(CONFIG_DIR)/config.toml ]; then \
		echo "Creating default config.toml..."; \
		printf '# Nexus Runtime Configuration\n\n' > $(CONFIG_DIR)/config.toml; \
		printf '[kernel]\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'mailbox_capacity = 1024\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'send_timeout_ms = 500\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'handle_timeout_ms = 30000\n\n' >> $(CONFIG_DIR)/config.toml; \
		printf '[logging]\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'level = "info"\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'format = "compact"\n\n' >> $(CONFIG_DIR)/config.toml; \
		printf '[runtime]\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'worker_threads = 0  # 0 = auto-detect CPU count\n' >> $(CONFIG_DIR)/config.toml; \
		printf 'max_blocking_threads = 512\n' >> $(CONFIG_DIR)/config.toml; \
		echo "$(SUCCESS_COLOR)Created $(CONFIG_DIR)/config.toml$(NO_COLOR)"; \
	else \
		echo "$(INFO_COLOR)Config file already exists$(NO_COLOR)"; \
	fi

## build: Build release binary
build: deps
	@echo "$(INFO_COLOR)Building Nexus $(VERSION) in release mode...$(NO_COLOR)"
	$(CARGO) build $(CARGO_FLAGS)
	@echo "$(SUCCESS_COLOR)Build complete: $(BUILD_DIR)/$(BINARY_NAME)$(NO_COLOR)"

## install: Build and install binary
install: build setup-dirs setup-config
	@echo "$(INFO_COLOR)Installing Nexus to $(INSTALL_DIR)...$(NO_COLOR)"
	@cp $(BUILD_DIR)/nexus-demo $(INSTALL_DIR)/$(BINARY_NAME)
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "$(SUCCESS_COLOR)Installed successfully!$(NO_COLOR)"
	@echo ""
	@echo "To run Nexus, ensure $(INSTALL_DIR) is in your PATH:"
	@echo "  export PATH=\"$(INSTALL_DIR):\$$PATH\""
	@echo ""
	@echo "Then run:"
	@echo "  $(BINARY_NAME)"

## run: Run Nexus with default configuration
run: build
	@echo "$(INFO_COLOR)Starting Nexus...$(NO_COLOR)"
	@RUST_LOG=$(RUST_LOG) $(CARGO) run --bin nexus-demo $(CARGO_FLAGS)

## test: Run unit and integration tests
test:
	@echo "$(INFO_COLOR)Running test suite...$(NO_COLOR)"
	$(CARGO) test
	@echo "$(SUCCESS_COLOR)All tests passed$(NO_COLOR)"

## test-all: Run all tests including ignored ones
test-all:
	@echo "$(INFO_COLOR)Running comprehensive test suite...$(NO_COLOR)"
	$(CARGO) test -- --include-ignored
	@echo "$(SUCCESS_COLOR)All tests passed$(NO_COLOR)"

## bench: Run benchmarks
bench:
	@echo "$(INFO_COLOR)Running benchmarks...$(NO_COLOR)"
	$(CARGO) bench

## fmt: Format code
fmt:
	@echo "$(INFO_COLOR)Formatting code...$(NO_COLOR)"
	$(CARGO) fmt
	@echo "$(SUCCESS_COLOR)Code formatted$(NO_COLOR)"

## clippy: Run linter with warnings as errors
clippy:
	@echo "$(INFO_COLOR)Running clippy...$(NO_COLOR)"
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	@echo "$(SUCCESS_COLOR)Clippy checks passed$(NO_COLOR)"

## verify: Verify installation
verify:
	@echo "$(INFO_COLOR)Verifying installation...$(NO_COLOR)"
	@if [ -f $(INSTALL_DIR)/$(BINARY_NAME) ]; then \
		echo "$(SUCCESS_COLOR)✓ Binary found: $(INSTALL_DIR)/$(BINARY_NAME)$(NO_COLOR)"; \
	else \
		echo "$(ERROR_COLOR)✗ Binary not found$(NO_COLOR)"; \
		exit 1; \
	fi
	@if [ -f $(CONFIG_DIR)/config.toml ]; then \
		echo "$(SUCCESS_COLOR)✓ Config found: $(CONFIG_DIR)/config.toml$(NO_COLOR)"; \
	else \
		echo "$(ERROR_COLOR)✗ Config not found$(NO_COLOR)"; \
		exit 1; \
	fi
	@if [ -d $(DATA_DIR) ]; then \
		echo "$(SUCCESS_COLOR)✓ Data directory: $(DATA_DIR)$(NO_COLOR)"; \
	else \
		echo "$(ERROR_COLOR)✗ Data directory not found$(NO_COLOR)"; \
		exit 1; \
	fi
	@echo "$(SUCCESS_COLOR)Installation verified successfully$(NO_COLOR)"

## doctor: Check development environment
doctor:
	@echo "$(INFO_COLOR)Checking development environment...$(NO_COLOR)"
	@echo ""
	@echo "Rust Toolchain:"
	@rustc --version 2>/dev/null || echo "$(ERROR_COLOR)  ✗ rustc not found$(NO_COLOR)"
	@cargo --version 2>/dev/null || echo "$(ERROR_COLOR)  ✗ cargo not found$(NO_COLOR)"
	@echo ""
	@echo "Project Status:"
	@echo "  Version: $(VERSION)"
	@echo "  Binary:  $(BINARY_NAME)"
	@echo "  Build:   $(BUILD_DIR)"
	@echo ""
	@echo "Paths:"
	@echo "  Install: $(INSTALL_DIR)"
	@echo "  Config:  $(CONFIG_DIR)"
	@echo "  Data:    $(DATA_DIR)"
	@echo ""
	@if command -v $(INSTALL_DIR)/$(BINARY_NAME) >/dev/null 2>&1; then \
		echo "$(SUCCESS_COLOR)✓ Nexus is installed and in PATH$(NO_COLOR)"; \
	elif [ -f $(INSTALL_DIR)/$(BINARY_NAME) ]; then \
		echo "$(INFO_COLOR)⚠ Nexus is installed but not in PATH$(NO_COLOR)"; \
	else \
		echo "$(INFO_COLOR)ℹ Nexus not installed (run 'make install')$(NO_COLOR)"; \
	fi

## deploy: Full production deployment
deploy: clean build test install verify
	@echo "$(SUCCESS_COLOR)Deployment complete!$(NO_COLOR)"
	@echo ""
	@echo "Nexus is ready for production use."
	@echo "Configuration: $(CONFIG_DIR)/config.toml"
	@echo "Logs: $(DATA_DIR)/logs"

## clean: Remove build artifacts
clean:
	@echo "$(INFO_COLOR)Cleaning build artifacts...$(NO_COLOR)"
	$(CARGO) clean
	@echo "$(SUCCESS_COLOR)Clean complete$(NO_COLOR)"

## distclean: Remove all generated files including installation
distclean: clean
	@echo "$(INFO_COLOR)Removing installation...$(NO_COLOR)"
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@rm -rf $(CONFIG_DIR)
	@rm -rf $(DATA_DIR)
	@echo "$(SUCCESS_COLOR)Full cleanup complete$(NO_COLOR)"

## uninstall: Remove installed files (keeps config and data)
uninstall:
	@echo "$(INFO_COLOR)Uninstalling Nexus...$(NO_COLOR)"
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "$(SUCCESS_COLOR)Uninstalled (config and data preserved)$(NO_COLOR)"
	@echo "To remove config: rm -rf $(CONFIG_DIR)"
	@echo "To remove data: rm -rf $(DATA_DIR)"
