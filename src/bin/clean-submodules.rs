use std::path::PathBuf;
use std::process::Command;

fn main() {
    let nim_codex_dir = PathBuf::from("vendor/nim-codex");

    println!("Cleaning cargo artifacts...");
    let status = Command::new("cargo")
        .arg("clean")
        .status()
        .expect("Failed to run cargo clean");

    if !status.success() {
        eprintln!("Warning: cargo clean failed");
    }

    if nim_codex_dir.exists() {
        println!("Deinitializing git submodules...");
        let status = Command::new("git")
            .args(&[
                "submodule",
                "deinit",
                "-f",
                &nim_codex_dir.to_string_lossy(),
            ])
            .status();

        match status {
            Ok(s) if s.success() => println!("Git submodules deinitialized successfully"),
            Ok(_) => eprintln!("Warning: Failed to deinitialize git submodules"),
            Err(e) => eprintln!("Warning: Could not run git command: {}", e),
        }
    }

    println!("Clean completed!");
}
