//! Build script for QAGML CUDA kernel compilation

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/cuda/qagml_kernel.cu");
    
    // Check if CUDA is available
    let cuda_available = check_cuda_installation();
    
    if !cuda_available {
        println!("cargo:warning=CUDA not found. Building without GPU support.");
        println!("cargo:warning=Install CUDA Toolkit 12.x for RTX 5090 support.");
        return;
    }
    
    println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
    println!("cargo:rustc-link-lib=cudart");
    println!("cargo:rustc-link-lib=cuda");
    
    // Compile CUDA kernel
    compile_cuda_kernel();
}

fn check_cuda_installation() -> bool {
    // Check for nvcc compiler
    Command::new("nvcc")
        .arg("--version")
        .output()
        .is_ok()
}

fn compile_cuda_kernel() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cuda_file = "src/cuda/qagml_kernel.cu";
    let output_file = out_dir.join("libqagml_cuda.a");
    
    println!("Compiling CUDA kernel: {}", cuda_file);
    
    // Compile CUDA kernel with nvcc
    let status = Command::new("nvcc")
        .args(&[
            "-O3",                          // Optimization level 3
            "-arch=sm_89",                  // RTX 5090 architecture (Ada Lovelace)
            "-gencode=arch=compute_89,code=sm_89",
            "--compiler-options", "-fPIC", // Position independent code
            "-c",                           // Compile only
            cuda_file,
            "-o",
            out_dir.join("qagml_kernel.o").to_str().unwrap(),
        ])
        .status();
    
    match status {
        Ok(status) if status.success() => {
            println!("✅ CUDA kernel compiled successfully");
            
            // Create static library
            Command::new("ar")
                .args(&[
                    "rcs",
                    output_file.to_str().unwrap(),
                    out_dir.join("qagml_kernel.o").to_str().unwrap(),
                ])
                .status()
                .expect("Failed to create static library");
            
            println!("cargo:rustc-link-search=native={}", out_dir.display());
            println!("cargo:rustc-link-lib=static=qagml_cuda");
        }
        Ok(status) => {
            panic!("CUDA compilation failed with status: {}", status);
        }
        Err(e) => {
            panic!("Failed to run nvcc: {}", e);
        }
    }
}

