use std::path::PathBuf;

/// Links against the prebuilt static library
pub fn link_prebuilt_library(lib_dir: &PathBuf) {
    println!("  [LINKER] Starting link_prebuilt_library");
    println!("  [LINKER] Library directory: {}", lib_dir.display());

    // Verify library directory exists
    if lib_dir.exists() {
        println!("  [LINKER] ✓ Library directory exists");

        // List files in the directory
        if let Ok(entries) = std::fs::read_dir(lib_dir) {
            println!("  [LINKER] Files in library directory:");
            for entry in entries.flatten() {
                let path = entry.path();
                let metadata = entry.metadata();
                if let Ok(meta) = metadata {
                    let size = meta.len();
                    let file_type = if meta.is_file() {
                        "file"
                    } else if meta.is_dir() {
                        "dir"
                    } else {
                        "other"
                    };
                    println!(
                        "  [LINKER]   - {} ({}, {} bytes)",
                        path.display(),
                        file_type,
                        size
                    );
                } else {
                    println!("  [LINKER]   - {} (metadata unavailable)", path.display());
                }
            }
        }
    } else {
        println!("  [LINKER] ✗ Library directory does not exist!");
    }

    println!("  [LINKER] Setting link search path...");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("  [LINKER] ✓ Link search path set");

    // Link each library separately
    println!("  [LINKER] Linking static libraries:");
    println!("  [LINKER]   - storage");
    println!("cargo:rustc-link-lib=static=storage");
    println!("  [LINKER]   - natpmp");
    println!("cargo:rustc-link-lib=static=natpmp");
    println!("  [LINKER]   - miniupnpc");
    println!("cargo:rustc-link-lib=static=miniupnpc");
    println!("  [LINKER]   - backtrace");
    println!("cargo:rustc-link-lib=static=backtrace");
    println!("  [LINKER]   - plum");
    println!("cargo:rustc-link-lib=static=plum");
    println!("  [LINKER] ✓ Static libraries linked");

    // System libraries required by the prebuilt library
    println!("  [LINKER] Linking system libraries:");
    println!("  [LINKER]   - stdc++");
    println!("cargo:rustc-link-lib=stdc++");
    println!("  [LINKER]   - gomp (dylib)");
    println!("cargo:rustc-link-lib=dylib=gomp");
    println!("  [LINKER] ✓ System libraries linked");

    // Linker flags
    println!("  [LINKER] Setting linker flags:");
    println!("  [LINKER]   - --allow-multiple-definition");
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("  [LINKER]   - --defsym=__rust_probestack=0");
    println!("cargo:rustc-link-arg=-Wl,--defsym=__rust_probestack=0");
    println!("  [LINKER]   - --whole-archive for static libraries");
    println!("cargo:rustc-link-arg=-Wl,--whole-archive");
    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
    println!("  [LINKER] ✓ Linker flags set");

    println!("  [LINKER] ✓ link_prebuilt_library completed successfully");
}
