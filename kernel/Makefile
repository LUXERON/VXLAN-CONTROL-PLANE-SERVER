# SYMMETRIX CORE KERNEL MODULE MAKEFILE v2.0
# Revolutionary Mathematical Operating System with Memory Shortage Illusion

# Module names
obj-m := symmetrix-core.o symmetrix-msi-fpga.o

# Source files
symmetrix-core-objs := symmetrix-core.o symmetrix-galois.o symmetrix-tensor.o symmetrix-sheaf.o
symmetrix-msi-fpga-objs := symmetrix-msi-fpga.o

# Kernel build directory
KERNEL_DIR ?= /lib/modules/$(shell uname -r)/build

# Current directory
PWD := $(shell pwd)

# Compiler flags
ccflags-y := -DCONFIG_SYMMETRIX_DEBUG -O3 -march=native

# Build targets
all: modules

modules:
	@echo "🔧 Building SYMMETRIX CORE kernel module..."
	$(MAKE) -C $(KERNEL_DIR) M=$(PWD) modules
	@echo "✅ Kernel module build completed"

clean:
	@echo "🧹 Cleaning kernel module build artifacts..."
	$(MAKE) -C $(KERNEL_DIR) M=$(PWD) clean
	rm -f *.symvers *.order *.mod.c
	@echo "✅ Clean completed"

install: modules
	@echo "📦 Installing SYMMETRIX CORE kernel module..."
	sudo $(MAKE) -C $(KERNEL_DIR) M=$(PWD) modules_install
	sudo depmod -a
	@echo "✅ Kernel module installed"

load: modules
	@echo "🚀 Loading SYMMETRIX v2.0 with Memory Shortage Illusion..."
	@echo "📦 Loading MSI FPGA driver..."
	sudo insmod symmetrix-msi-fpga.ko
	@echo "📦 Loading SYMMETRIX CORE module..."
	sudo insmod symmetrix-core.ko max_containers=5000 enable_tensor_allocator=1 enable_sheaf_scheduler=1 enable_msi=1 msi_amplification_ratio=125000
	@echo "✅ SYMMETRIX v2.0 loaded successfully"
	@echo "📊 Check status: cat /proc/symmetrix/status"

unload:
	@echo "🛑 Unloading SYMMETRIX v2.0 modules..."
	sudo rmmod symmetrix-core || true
	sudo rmmod symmetrix-msi-fpga || true
	@echo "✅ SYMMETRIX modules unloaded"

reload: unload load

status:
	@echo "📊 SYMMETRIX CORE Kernel Module Status:"
	@if lsmod | grep -q symmetrix; then \
		echo "✅ Module loaded"; \
		if [ -f /proc/symmetrix/status ]; then \
			echo "📋 Status:"; \
			cat /proc/symmetrix/status; \
		fi; \
	else \
		echo "❌ Module not loaded"; \
	fi

test: load
	@echo "🧪 Testing SYMMETRIX CORE kernel module..."
	@echo "📊 Module status:"
	@cat /proc/symmetrix/status
	@echo ""
	@echo "🔍 Kernel log (last 10 lines):"
	@dmesg | grep symmetrix | tail -10
	@echo "✅ Test completed"

debug:
	@echo "🐛 SYMMETRIX CORE Debug Information:"
	@echo "📋 Module info:"
	@modinfo symmetrix-core.ko 2>/dev/null || echo "Module not built"
	@echo ""
	@echo "📊 System info:"
	@echo "  Kernel: $(shell uname -r)"
	@echo "  Architecture: $(shell uname -m)"
	@echo "  CPUs: $(shell nproc)"
	@echo "  Memory: $(shell free -h | grep Mem | awk '{print $$2}')"
	@echo ""
	@echo "🔍 Recent kernel messages:"
	@dmesg | grep -i symmetrix | tail -5 || echo "No symmetrix messages found"

benchmark: load
	@echo "⚡ Running kernel module benchmarks..."
	@echo "🧮 Testing mathematical operations..."
	@echo "This would run kernel-level performance tests"
	@echo "📊 Results would be logged to kernel messages"

help:
	@echo "🌟 SYMMETRIX CORE Kernel Module Build System"
	@echo "Revolutionary Mathematical Operating System"
	@echo ""
	@echo "Available targets:"
	@echo "  all        - Build kernel module (default)"
	@echo "  modules    - Build kernel module"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install module to system"
	@echo "  load       - Load module into kernel"
	@echo "  unload     - Unload module from kernel"
	@echo "  reload     - Unload and reload module"
	@echo "  status     - Show module status"
	@echo "  test       - Load module and run basic tests"
	@echo "  debug      - Show debug information"
	@echo "  benchmark  - Run performance benchmarks"
	@echo "  help       - Show this help"
	@echo ""
	@echo "🔧 Module parameters:"
	@echo "  max_containers=N        - Maximum containers (default: 5000)"
	@echo "  enable_tensor_allocator=1/0 - Enable tensor allocator (default: 1)"
	@echo "  enable_sheaf_scheduler=1/0  - Enable sheaf scheduler (default: 1)"
	@echo "  galois_prime_exp=N      - Galois prime exponent (default: 61)"
	@echo ""
	@echo "📊 Usage examples:"
	@echo "  make load max_containers=10000"
	@echo "  make test"
	@echo "  make status"

.PHONY: all modules clean install load unload reload status test debug benchmark help
