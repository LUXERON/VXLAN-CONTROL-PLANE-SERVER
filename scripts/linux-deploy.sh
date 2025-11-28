#!/bin/bash

# SYMMETRIX LINUX DEPLOYMENT SCRIPT
# Revolutionary Mathematical Operating System - Linux Kernel Module Deployment
# "Transform Linux into a Mathematical Supercomputer"

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_header() {
    echo -e "${PURPLE}$1${NC}"
}

# Check if running as root for kernel operations
check_root() {
    if [[ $EUID -eq 0 ]]; then
        log_warning "Running as root - kernel module operations enabled"
        return 0
    else
        log_info "Not running as root - some operations may require sudo"
        return 1
    fi
}

# Check system requirements
check_requirements() {
    log_header "🔍 Checking System Requirements"
    
    # Check kernel headers
    if [ -d "/lib/modules/$(uname -r)/build" ]; then
        log_success "Kernel headers found: $(uname -r)"
    else
        log_error "Kernel headers not found. Install with:"
        log_info "  Ubuntu/Debian: sudo apt-get install linux-headers-$(uname -r)"
        log_info "  RHEL/CentOS: sudo yum install kernel-devel"
        log_info "  Arch: sudo pacman -S linux-headers"
        exit 1
    fi
    
    # Check build tools
    local missing_tools=()
    for tool in make gcc; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing build tools: ${missing_tools[*]}"
        log_info "Install with: sudo apt-get install build-essential"
        exit 1
    fi
    
    log_success "All requirements satisfied"
}

# Build kernel module
build_kernel_module() {
    log_header "🔧 Building SYMMETRIX Kernel Module"
    
    if [ ! -f "Makefile" ]; then
        log_error "Kernel Makefile not found. Are you in the kernel directory?"
        exit 1
    fi
    
    log_info "Cleaning previous builds..."
    make clean
    
    log_info "Building kernel module..."
    if make; then
        log_success "Kernel module built successfully"
    else
        log_error "Kernel module build failed"
        exit 1
    fi
    
    # Verify module was built
    if [ -f "symmetrix-core.ko" ]; then
        log_success "Module file created: symmetrix-core.ko"
        log_info "Module info:"
        modinfo symmetrix-core.ko
    else
        log_error "Module file not found after build"
        exit 1
    fi
}

# Load kernel module
load_kernel_module() {
    log_header "🚀 Loading SYMMETRIX Kernel Module"
    
    # Check if module is already loaded
    if lsmod | grep -q symmetrix; then
        log_warning "SYMMETRIX module already loaded. Unloading first..."
        sudo rmmod symmetrix-core || true
    fi
    
    log_info "Loading kernel module..."
    if sudo insmod symmetrix-core.ko max_containers=5000 enable_tensor_allocator=1 enable_sheaf_scheduler=1; then
        log_success "Kernel module loaded successfully"
    else
        log_error "Failed to load kernel module"
        exit 1
    fi
    
    # Verify module is loaded
    if lsmod | grep -q symmetrix; then
        log_success "Module verification: LOADED"
    else
        log_error "Module verification: NOT LOADED"
        exit 1
    fi
    
    # Check proc interface
    if [ -f "/proc/symmetrix/status" ]; then
        log_success "Proc interface available: /proc/symmetrix/status"
        log_info "System status:"
        cat /proc/symmetrix/status
    else
        log_warning "Proc interface not found"
    fi
}

# Install user-space binaries
install_binaries() {
    log_header "📦 Installing SYMMETRIX Binaries"
    
    # Create directories
    sudo mkdir -p /opt/symmetrix/bin
    sudo mkdir -p /etc/symmetrix
    sudo mkdir -p /var/lib/symmetrix
    sudo mkdir -p /var/log/symmetrix
    
    # Install binaries if they exist
    if [ -d "bin" ] && [ "$(ls -A bin/)" ]; then
        log_info "Installing binaries..."
        sudo cp bin/symmetrix-* /opt/symmetrix/bin/ 2>/dev/null || true
        sudo chmod +x /opt/symmetrix/bin/*
        
        # Create symlinks
        sudo ln -sf /opt/symmetrix/bin/symmetrix-cli /usr/local/bin/symmetrix-cli 2>/dev/null || true
        
        log_success "Binaries installed to /opt/symmetrix/bin/"
    else
        log_warning "No binaries found in bin/ directory"
    fi
    
    # Install configuration
    if [ -f "symmetrix.toml" ]; then
        sudo cp symmetrix.toml /etc/symmetrix/
        log_success "Configuration installed"
    fi
}

# Create systemd service
create_service() {
    log_header "⚙️ Creating Systemd Service"
    
    if [ -f "/opt/symmetrix/bin/symmetrix-daemon" ]; then
        cat << EOF | sudo tee /etc/systemd/system/symmetrix-daemon.service > /dev/null
[Unit]
Description=SYMMETRIX Mathematical Operating System Daemon
Documentation=https://symmetrix.dev
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
ExecStart=/opt/symmetrix/bin/symmetrix-daemon
ExecReload=/bin/kill -HUP \$MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=symmetrix-daemon

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/symmetrix /var/log/symmetrix

[Install]
WantedBy=multi-user.target
EOF
        
        sudo systemctl daemon-reload
        sudo systemctl enable symmetrix-daemon
        log_success "Systemd service created and enabled"
    else
        log_warning "Daemon binary not found, skipping service creation"
    fi
}

# Run system tests
run_tests() {
    log_header "🧪 Running System Tests"
    
    # Test kernel module
    if lsmod | grep -q symmetrix; then
        log_success "✅ Kernel module: LOADED"
    else
        log_error "❌ Kernel module: NOT LOADED"
        return 1
    fi
    
    # Test proc interface
    if [ -f "/proc/symmetrix/status" ]; then
        log_success "✅ Proc interface: AVAILABLE"
    else
        log_error "❌ Proc interface: NOT AVAILABLE"
        return 1
    fi
    
    # Test CLI
    if command -v symmetrix-cli &> /dev/null; then
        log_success "✅ CLI tool: AVAILABLE"
        if symmetrix-cli system info &> /dev/null; then
            log_success "✅ CLI functionality: WORKING"
        else
            log_warning "⚠️ CLI functionality: LIMITED"
        fi
    else
        log_warning "⚠️ CLI tool: NOT AVAILABLE"
    fi
    
    # Test mathematical operations
    log_info "Testing mathematical acceleration..."
    if [ -f "/proc/symmetrix/status" ]; then
        if grep -q "Mathematical Operations: Accelerated" /proc/symmetrix/status; then
            log_success "✅ Mathematical acceleration: ACTIVE"
        else
            log_warning "⚠️ Mathematical acceleration: STATUS UNKNOWN"
        fi
    fi
    
    log_success "System tests completed"
}

# Show system status
show_status() {
    log_header "📊 SYMMETRIX System Status"
    
    echo "═══════════════════════════════════════════════════════════════"
    echo "🌟 SYMMETRIX MATHEMATICAL OPERATING SYSTEM"
    echo "Revolutionary Linux Kernel Integration"
    echo "═══════════════════════════════════════════════════════════════"
    echo
    
    echo "🔧 Kernel Module:"
    if lsmod | grep -q symmetrix; then
        echo "  Status: ✅ LOADED"
        echo "  Module: $(lsmod | grep symmetrix)"
    else
        echo "  Status: ❌ NOT LOADED"
    fi
    echo
    
    echo "📊 System Interface:"
    if [ -f "/proc/symmetrix/status" ]; then
        echo "  Proc Interface: ✅ AVAILABLE"
        echo "  Status File: /proc/symmetrix/status"
        echo
        echo "📋 Current Status:"
        cat /proc/symmetrix/status | sed 's/^/  /'
    else
        echo "  Proc Interface: ❌ NOT AVAILABLE"
    fi
    echo
    
    echo "🛠️ User Tools:"
    if command -v symmetrix-cli &> /dev/null; then
        echo "  CLI Tool: ✅ AVAILABLE (/usr/local/bin/symmetrix-cli)"
    else
        echo "  CLI Tool: ❌ NOT AVAILABLE"
    fi
    
    if [ -f "/opt/symmetrix/bin/symmetrix-daemon" ]; then
        echo "  Daemon: ✅ INSTALLED"
        if systemctl is-active --quiet symmetrix-daemon; then
            echo "  Service: ✅ RUNNING"
        else
            echo "  Service: ⏸️ STOPPED"
        fi
    else
        echo "  Daemon: ❌ NOT INSTALLED"
    fi
    echo
    
    echo "🌐 Next Steps:"
    echo "  • View status: cat /proc/symmetrix/status"
    echo "  • Use CLI: symmetrix-cli --help"
    echo "  • Start daemon: sudo systemctl start symmetrix-daemon"
    echo "  • Monitor logs: journalctl -u symmetrix-daemon -f"
    echo
    echo "🎉 Linux transformed into mathematical supercomputer!"
    echo "═══════════════════════════════════════════════════════════════"
}

# Main deployment function
main() {
    log_header "🚀 SYMMETRIX LINUX DEPLOYMENT"
    log_header "Transform Linux into a Mathematical Supercomputer"
    echo
    
    # Check if we're in the right directory
    if [ ! -f "symmetrix-core.c" ] && [ ! -f "Makefile" ]; then
        log_error "Not in kernel module directory. Please cd to the kernel/ directory first."
        exit 1
    fi
    
    check_requirements
    build_kernel_module
    load_kernel_module
    install_binaries
    create_service
    run_tests
    show_status
    
    log_success "🎉 SYMMETRIX deployment completed successfully!"
    log_info "Your Linux system is now a mathematical supercomputer! 🌟"
}

# Handle command line arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "status")
        show_status
        ;;
    "test")
        run_tests
        ;;
    "unload")
        log_info "Unloading SYMMETRIX kernel module..."
        sudo rmmod symmetrix-core || log_error "Failed to unload module"
        ;;
    "reload")
        log_info "Reloading SYMMETRIX kernel module..."
        sudo rmmod symmetrix-core || true
        sudo insmod symmetrix-core.ko max_containers=5000 enable_tensor_allocator=1 enable_sheaf_scheduler=1
        log_success "Module reloaded"
        ;;
    *)
        echo "Usage: $0 [deploy|status|test|unload|reload]"
        echo "  deploy  - Full deployment (default)"
        echo "  status  - Show system status"
        echo "  test    - Run system tests"
        echo "  unload  - Unload kernel module"
        echo "  reload  - Reload kernel module"
        exit 1
        ;;
esac
