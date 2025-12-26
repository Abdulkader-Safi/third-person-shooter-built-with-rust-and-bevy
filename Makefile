.PHONY: build run web serve clean setup help dist-web

# Default target
.DEFAULT_GOAL := help

# Show available commands
help:
	@echo "Available commands:"
	@echo "  make build    - Build native release"
	@echo "  make run      - Run the game in development mode"
	@echo "  make web      - Build for WebAssembly (outputs to dist/)"
	@echo "  make dist-web - Build for WebAssembly and create zip for itch.io"
	@echo "  make serve    - Serve web build locally at http://127.0.0.1:8080"
	@echo "  make clean    - Clean all build artifacts"
	@echo "  make setup    - Install dependencies for web builds"
	@echo "  make help     - Show this help message"

# Build native release
build:
	cargo build --release

# Run the game in development mode
run:
	cargo run

# Build for WebAssembly (release)
web:
	trunk build --release

# Build for WebAssembly and create zip for itch.io
dist-web:
	trunk build --release --public-url ./
	cd dist && zip -r ../game.zip .
	@echo "Created game.zip ready for itch.io"

# Serve web build locally for testing
serve:
	trunk serve

# Clean all build artifacts
clean:
	cargo clean
	rm -rf dist
	rm -f my_bevy_game-web.zip

# Setup dependencies for web builds
setup:
	rustup target add wasm32-unknown-unknown
	cargo install trunk
