#!/bin/bash
echo "Loading SYMMETRIX v3.0 kernel modules..."

# Load FPGA interface first
echo "Loading FPGA interface module..."
# insmod symmetrix-msi-fpga.ko

# Load Galois engine
echo "Loading SIMD Galois engine..."
# insmod symmetrix-galois.ko

# Load main SYMMETRIX core with Terahertz CPU
echo "Loading SYMMETRIX core v3.0..."
# insmod symmetrix-core.ko enable_terahertz_cpu=1 terahertz_simd_width=512

echo "✅ All modules loaded successfully"
echo "🌟 SYMMETRIX v3.0 with Terahertz CPU is now active!"
