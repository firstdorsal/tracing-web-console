use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let frontend_dir = Path::new("frontend");
    let dist_dir = frontend_dir.join("dist");

    // Track all frontend source files for rebuilds
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=frontend/tsconfig.json");
    println!("cargo:rerun-if-changed=frontend/tailwind.config.js");
    println!("cargo:rerun-if-changed=frontend/index.html");

    // Recursively track all files in src directory
    track_directory("frontend/src");

    // If dist already exists (e.g., during cargo publish), skip building
    // This is necessary because cargo publish verifies the package in a temp directory
    // and build scripts cannot modify the source directory
    if dist_dir.exists() && dist_dir.join("index.html").exists() {
        println!("cargo:warning=Frontend dist already exists, skipping build");
        println!("cargo:rerun-if-changed=frontend/dist");
        return;
    }

    // Check if we should build the frontend
    if frontend_dir.exists() {
        println!("cargo:warning=Building frontend...");

        // Install dependencies if node_modules doesn't exist
        if !frontend_dir.join("node_modules").exists() {
            println!("cargo:warning=Installing frontend dependencies...");
            let install_status = Command::new("pnpm")
                .args(["install"])
                .current_dir(frontend_dir)
                .status();

            match install_status {
                Ok(status) if status.success() => {
                    println!("cargo:warning=Frontend dependencies installed successfully");
                }
                Ok(status) => {
                    println!("cargo:warning=pnpm install failed with status: {}", status);
                    println!("cargo:warning=Frontend will use placeholder page");
                    return;
                }
                Err(e) => {
                    println!("cargo:warning=Failed to run pnpm install: {}", e);
                    println!("cargo:warning=Make sure pnpm is installed");
                    println!("cargo:warning=Frontend will use placeholder page");
                    return;
                }
            }
        }

        // Build the frontend
        println!("cargo:warning=Running pnpm build...");
        let build_status = Command::new("pnpm")
            .args(["build"])
            .current_dir(frontend_dir)
            .status();

        match build_status {
            Ok(status) if status.success() => {
                println!("cargo:warning=Frontend built successfully");
                if dist_dir.exists() {
                    println!("cargo:rerun-if-changed=frontend/dist");
                }
            }
            Ok(status) => {
                println!("cargo:warning=pnpm build failed with status: {}", status);
                println!("cargo:warning=Frontend will use placeholder page");
            }
            Err(e) => {
                println!("cargo:warning=Failed to run pnpm build: {}", e);
                println!("cargo:warning=Make sure pnpm is installed");
                println!("cargo:warning=Frontend will use placeholder page");
            }
        }
    } else {
        println!("cargo:warning=Frontend directory not found");
        println!("cargo:warning=The tracing UI will show a placeholder page");
    }
}

fn track_directory(dir: &str) {
    let path = Path::new(dir);
    if !path.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    track_directory(path_str);
                }
            } else if let Some(path_str) = path.to_str() {
                println!("cargo:rerun-if-changed={}", path_str);
            }
        }
    }
}
