use std::process::Command;

fn main() {
    // Rerun if frontend source changes
    println!("cargo:rerun-if-changed=frontend/main.js");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/node_modules/.package-lock.json");

    // Emit build timestamp for cache busting
    println!(
        "cargo:rustc-env=BUILD_TIME={}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Run esbuild to bundle main.js → src/frontend/terminal.js
    let esbuild = "frontend/node_modules/.bin/esbuild";
    let status = Command::new(esbuild)
        .args([
            "frontend/main.js",
            "--bundle",
            "--outfile=src/frontend/terminal.js",
        ])
        .status()
        .unwrap_or_else(|_| {
            panic!(
                "esbuild not found at {esbuild}. Run `npm install` in the terminal/frontend directory."
            )
        });

    if !status.success() {
        panic!("esbuild failed — check frontend/main.js for errors");
    }
}
