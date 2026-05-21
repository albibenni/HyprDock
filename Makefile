.PHONY: all build release run debug test coverage stop clean help

# Environment variables for debugging
DEBUG_ENV ?= GTK_DEBUG=interactive GDK_DEBUG=events G_MESSAGES_DEBUG=all RUST_LOG=hyprdock=debug

all: build

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

# Run with GTK inspector and verbose logging
debug:
	$(DEBUG_ENV) cargo run

test:
	cargo test

coverage:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || (echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; exit 1)
	@command -v jq >/dev/null 2>&1 || (echo "jq is required. Install it with your package manager (e.g. sudo apt install jq)."; exit 1)
	@command -v column >/dev/null 2>&1 || (echo "column is required (usually provided by util-linux/bsdextrautils)."; exit 1)
	@tmp_file="$$(mktemp)"; \
	cargo llvm-cov --workspace --all-features --json --summary-only --output-path "$$tmp_file" -- --test-threads=1; \
	jq -r '"File\tLines %\tRegions %\tFunctions %", (.data[0].files[] | "\(.filename)\t\(.summary.lines.percent // 0)\t\(.summary.regions.percent // 0)\t\(.summary.functions.percent // 0)"), "TOTAL\t\(.data[0].totals.lines.percent // 0)\t\(.data[0].totals.regions.percent // 0)\t\(.data[0].totals.functions.percent // 0)"' "$$tmp_file" | column -t -s "$$(printf '\t')"; \
	rm -f "$$tmp_file"

stop:
	@pkill -f "target/debug/HyprDock" 2>/dev/null || true
	@pkill -f "target/release/HyprDock" 2>/dev/null || true

clean:
	cargo clean

help:
	@echo "HyprDock Makefile"
	@echo "Usage: make <target>"
	@echo ""
	@echo "  build             Build the project (debug)"
	@echo "  release           Build the project (release)"
	@echo "  run               Run the dock"
	@echo "  debug             Run with GTK inspector and debug logs"
	@echo "  test              Run all tests"
	@echo "  coverage          Generate and print coverage summary"
	@echo "  stop              Kill any running HyprDock process"
	@echo "  clean             Remove build artifacts"
	@echo "  help              Show this message"
