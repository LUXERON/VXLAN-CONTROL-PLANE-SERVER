# SYMMETRIX CORE MAKEFILE
# Revolutionary Mathematical Operating System Build System

.PHONY: all build test bench clean install docker iso kernel help

# Default target
all: build

# Build configuration
CARGO_FLAGS := --release
RUST_LOG := info
TARGET_DIR := target/release

# Version information
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
BUILD_DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_HASH := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Build targets
build: ## Build all Symmetrix components
	@echo "🚀 Building SYMMETRIX CORE v$(VERSION)"
	@echo "📅 Build Date: $(BUILD_DATE)"
	@echo "🔗 Git Hash: $(GIT_HASH)"
	cargo build $(CARGO_FLAGS)
	@echo "✅ Build completed successfully"

build-dev: ## Build in development mode
	@echo "🛠️ Building SYMMETRIX CORE (development mode)"
	cargo build
	@echo "✅ Development build completed"

test: ## Run all tests
	@echo "🧪 Running SYMMETRIX CORE tests"
	cargo test --all
	@echo "✅ All tests passed"

test-math: ## Run mathematical engine tests
	@echo "🧮 Testing mathematical engines"
	cargo test --package symmetrix-sheaf
	cargo test --package symmetrix-galois  
	cargo test --package symmetrix-tensor
	@echo "✅ Mathematical tests passed"

bench: ## Run performance benchmarks
	@echo "📊 Running SYMMETRIX performance benchmarks"
	cargo run --bin symmetrix-benchmark -- all
	@echo "✅ Benchmarks completed"

bench-gpu: ## Run GPU comparison benchmarks
	@echo "🎮 Running GPU comparison benchmarks"
	cargo run --bin symmetrix-benchmark -- gpu-comparison --sizes=512,1024,2048,4096
	@echo "✅ GPU comparison completed"

bench-containers: ## Benchmark container orchestration
	@echo "🐳 Benchmarking container orchestration"
	cargo run --bin symmetrix-benchmark -- container-orchestration --containers=5000
	@echo "✅ Container benchmarks completed"

clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts"
	cargo clean
	rm -rf dist/
	rm -rf iso/
	@echo "✅ Clean completed"

install: build ## Install Symmetrix binaries
	@echo "📦 Installing SYMMETRIX CORE"
	sudo mkdir -p /opt/symmetrix/bin
	sudo mkdir -p /etc/symmetrix
	sudo mkdir -p /var/lib/symmetrix
	sudo mkdir -p /var/log/symmetrix
	
	# Install binaries
	sudo cp $(TARGET_DIR)/symmetrix-daemon /opt/symmetrix/bin/
	sudo cp $(TARGET_DIR)/symmetrix-benchmark /opt/symmetrix/bin/
	sudo cp $(TARGET_DIR)/symmetrix-compiler /opt/symmetrix/bin/
	sudo cp $(TARGET_DIR)/symmetrix-vm /opt/symmetrix/bin/
	
	# Install CLI with symlink
	sudo cp $(TARGET_DIR)/symmetrix-cli /opt/symmetrix/bin/
	sudo ln -sf /opt/symmetrix/bin/symmetrix-cli /usr/local/bin/symmetrix-cli
	
	# Install configuration
	sudo cp config/symmetrix.toml /etc/symmetrix/config.toml
	
	# Install systemd service
	sudo cp scripts/symmetrix-daemon.service /etc/systemd/system/
	sudo systemctl daemon-reload
	
	@echo "✅ Installation completed"
	@echo "🌐 Start daemon: sudo systemctl start symmetrix-daemon"
	@echo "🔧 CLI available: symmetrix-cli --help"

uninstall: ## Uninstall Symmetrix
	@echo "🗑️ Uninstalling SYMMETRIX CORE"
	sudo systemctl stop symmetrix-daemon 2>/dev/null || true
	sudo systemctl disable symmetrix-daemon 2>/dev/null || true
	sudo rm -f /etc/systemd/system/symmetrix-daemon.service
	sudo rm -rf /opt/symmetrix
	sudo rm -f /usr/local/bin/symmetrix-cli
	sudo systemctl daemon-reload
	@echo "✅ Uninstallation completed"

docker: ## Build Docker container
	@echo "🐳 Building SYMMETRIX Docker container"
	docker build -t symmetrix-core:$(VERSION) .
	docker tag symmetrix-core:$(VERSION) symmetrix-core:latest
	@echo "✅ Docker container built"
	@echo "🚀 Run: docker run -p 8080:8080 symmetrix-core:latest"

docker-run: docker ## Build and run Docker container
	@echo "🚀 Running SYMMETRIX in Docker"
	docker run -it --rm \
		-p 8080:8080 \
		-p 8443:8443 \
		--name symmetrix-core \
		symmetrix-core:latest

cross-linux: ## Cross-compile for Linux using cross + Podman
	@echo "🐧 Cross-compiling SYMMETRIX for Linux..."
	CROSS_CONTAINER_ENGINE=podman cross build --target x86_64-unknown-linux-musl --release
	@echo "✅ Linux binaries ready in target/x86_64-unknown-linux-musl/release/"

cross-kernel: cross-linux ## Prepare kernel module for Linux deployment
	@echo "🔧 Preparing kernel module package for Linux..."
	mkdir -p target/linux-deployment
	cp -r kernel/* target/linux-deployment/
	mkdir -p target/linux-deployment/bin
	cp target/x86_64-unknown-linux-musl/release/symmetrix-* target/linux-deployment/bin/ 2>/dev/null || true
	@echo "📦 Creating deployment script..."
	@echo '#!/bin/bash' > target/linux-deployment/deploy.sh
	@echo 'echo "🚀 SYMMETRIX LINUX DEPLOYMENT"' >> target/linux-deployment/deploy.sh
	@echo 'echo "Building and loading kernel module..."' >> target/linux-deployment/deploy.sh
	@echo 'make clean && make' >> target/linux-deployment/deploy.sh
	@echo 'sudo make load' >> target/linux-deployment/deploy.sh
	@echo 'echo "Installing binaries..."' >> target/linux-deployment/deploy.sh
	@echo 'sudo mkdir -p /opt/symmetrix/bin' >> target/linux-deployment/deploy.sh
	@echo 'sudo cp bin/symmetrix-* /opt/symmetrix/bin/' >> target/linux-deployment/deploy.sh
	@echo 'echo "✅ SYMMETRIX deployed! Check: cat /proc/symmetrix/status"' >> target/linux-deployment/deploy.sh
	chmod +x target/linux-deployment/deploy.sh
	@echo "✅ Kernel module package ready: target/linux-deployment/"
	@echo "📋 Deploy on Linux: cd target/linux-deployment && ./deploy.sh"

iso: build ## Build SymmetrixOS ISO
	@echo "💿 Building SymmetrixOS ISO"
	mkdir -p iso/boot iso/live iso/install
	
	# Copy kernel and initrd (placeholder - would be actual custom kernel)
	cp /boot/vmlinuz-$(shell uname -r) iso/boot/vmlinuz-symmetrix || echo "⚠️ Kernel copy failed (expected in dev)"
	cp /boot/initrd.img-$(shell uname -r) iso/boot/initrd-symmetrix || echo "⚠️ Initrd copy failed (expected in dev)"
	
	# Copy Symmetrix binaries
	mkdir -p iso/live/symmetrix/bin
	cp $(TARGET_DIR)/* iso/live/symmetrix/bin/ 2>/dev/null || true
	
	# Create filesystem
	mkdir -p iso/live/filesystem
	echo "SymmetrixOS Live System" > iso/live/filesystem/README
	
	# Create ISO (requires genisoimage)
	if command -v genisoimage >/dev/null 2>&1; then \
		genisoimage -o symmetrix-os-$(VERSION).iso \
			-b boot/grub/stage2_eltorito \
			-no-emul-boot \
			-boot-load-size 4 \
			-boot-info-table \
			-r -J -l -T \
			iso/; \
		echo "✅ SymmetrixOS ISO created: symmetrix-os-$(VERSION).iso"; \
	else \
		echo "⚠️ genisoimage not found - ISO creation skipped"; \
		echo "📦 Install: sudo apt-get install genisoimage"; \
	fi

kernel: ## Build custom kernel (placeholder)
	@echo "🔧 Building custom Symmetrix kernel"
	@echo "⚠️ Custom kernel build not yet implemented"
	@echo "📋 This would:"
	@echo "   - Download Linux kernel source"
	@echo "   - Apply Symmetrix mathematical patches"
	@echo "   - Configure with Symmetrix options"
	@echo "   - Build optimized kernel"
	@echo "   - Package kernel modules"

demo: build ## Run live demonstration
	@echo "🎭 Starting SYMMETRIX live demonstration"
	@echo "🚀 Launching daemon in background..."
	$(TARGET_DIR)/symmetrix-daemon &
	sleep 3
	
	@echo "📊 Running system info..."
	$(TARGET_DIR)/symmetrix-cli system info
	
	@echo "🧮 Testing mathematical engines..."
	$(TARGET_DIR)/symmetrix-cli math status
	
	@echo "🐳 Simulating container launch..."
	$(TARGET_DIR)/symmetrix-cli containers launch --template=demo --count=10
	
	@echo "📈 Running quick benchmark..."
	$(TARGET_DIR)/symmetrix-benchmark quick
	
	@echo "✅ Demo completed!"
	@echo "🌐 Web interface: http://localhost:8080"

dev-setup: ## Set up development environment
	@echo "🛠️ Setting up SYMMETRIX development environment"
	
	# Install Rust if not present
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "📦 Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; \
		source ~/.cargo/env; \
	fi
	
	# Install required tools
	cargo install cargo-watch
	cargo install cargo-audit
	cargo install cargo-outdated
	
	# Install system dependencies (Ubuntu/Debian)
	@if command -v apt-get >/dev/null 2>&1; then \
		echo "📦 Installing system dependencies..."; \
		sudo apt-get update; \
		sudo apt-get install -y build-essential pkg-config libssl-dev; \
	fi
	
	@echo "✅ Development environment ready"
	@echo "🔧 Start development: make watch"

watch: ## Watch for changes and rebuild
	@echo "👀 Watching for changes..."
	cargo watch -x "build" -x "test"

audit: ## Security audit
	@echo "🔒 Running security audit"
	cargo audit
	@echo "✅ Security audit completed"

update: ## Update dependencies
	@echo "📦 Updating dependencies"
	cargo update
	cargo outdated
	@echo "✅ Dependencies updated"

docs: ## Generate documentation
	@echo "📚 Generating documentation"
	cargo doc --all --no-deps
	@echo "✅ Documentation generated"
	@echo "🌐 View: cargo doc --open"

release: test bench ## Prepare release build
	@echo "🎉 Preparing SYMMETRIX CORE v$(VERSION) release"
	
	# Ensure clean state
	git status --porcelain | grep -q . && echo "❌ Working directory not clean" && exit 1 || true
	
	# Build release
	cargo build --release
	
	# Run full test suite
	cargo test --all --release
	
	# Run benchmarks
	cargo run --bin symmetrix-benchmark -- all --quick
	
	# Create release artifacts
	mkdir -p dist
	cp $(TARGET_DIR)/symmetrix-daemon dist/
	cp $(TARGET_DIR)/symmetrix-cli dist/
	cp $(TARGET_DIR)/symmetrix-benchmark dist/
	cp README.md dist/
	cp LICENSE* dist/
	
	# Create tarball
	tar -czf dist/symmetrix-core-$(VERSION)-linux-x86_64.tar.gz -C dist .
	
	@echo "✅ Release v$(VERSION) prepared"
	@echo "📦 Artifacts in dist/"

help: ## Show this help
	@echo "🌟 SYMMETRIX CORE BUILD SYSTEM"
	@echo "Revolutionary Mathematical Operating System"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "🚀 Quick start: make build && make demo"
	@echo "🌐 Full system: make install && sudo systemctl start symmetrix-daemon"
