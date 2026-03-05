use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=zig-wasm/src/");

    let zig_dir = std::path::Path::new("zig-wasm");
    if !zig_dir.exists() {
        eprintln!("Warning: zig-wasm directory not found, skipping WASM build");
        return;
    }

    let status = Command::new("zig")
        .args(["build", "-Doptimize=ReleaseSmall"])
        .current_dir(zig_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Zig WASM module built successfully");
        }
        Ok(s) => {
            eprintln!("Warning: Zig build exited with status {s}, continuing without WASM module");
        }
        Err(e) => {
            eprintln!("Warning: Could not run zig build: {e}, continuing without WASM module");
        }
    }
}
