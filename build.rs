use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=zig-wasm/src/");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to embed app icon resource: {e}");
        }
    }

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
