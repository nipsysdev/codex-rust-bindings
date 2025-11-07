use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
enum LinkingMode {
    Static,
    Dynamic,
}

fn determine_linking_mode() -> LinkingMode {
    let static_enabled = cfg!(feature = "static-linking");
    let dynamic_enabled = cfg!(feature = "dynamic-linking");

    match (static_enabled, dynamic_enabled) {
        (true, false) => LinkingMode::Static,
        (false, true) => LinkingMode::Dynamic,
        (false, false) => LinkingMode::Dynamic,
        (true, true) => {
            panic!("Cannot enable both 'static-linking' and 'dynamic-linking' features simultaneously. Please choose one.");
        }
    }
}

/// Ensure git submodules are initialized and updated
fn ensure_submodules(nim_codex_dir: &PathBuf) {
    if !nim_codex_dir.exists() {
        println!("cargo:warning=Initializing git submodules...");
        let status = Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .status()
            .expect("Failed to execute git command. Make sure git is installed and in PATH.");

        if !status.success() {
            panic!("Failed to initialize git submodules");
        }
        println!("cargo:warning=Git submodules initialized successfully");
    }
}

/// Build libcodex with static linking
fn build_libcodex_static(nim_codex_dir: &PathBuf) {
    println!("cargo:warning=Building libcodex with static linking...");

    // Get CODEX_LIB_PARAMS from environment if set
    let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

    let mut make_cmd = Command::new("make");
    make_cmd.args(&[
        "-C",
        &nim_codex_dir.to_string_lossy(),
        "STATIC=1",
        "libcodex",
    ]);

    // Add custom parameters if provided
    if !codex_params.is_empty() {
        make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
    }

    let status = make_cmd
        .status()
        .expect("Failed to execute make command. Make sure make is installed and in PATH.");

    if !status.success() {
        panic!("Failed to build libcodex with static linking");
    }

    println!("cargo:warning=libcodex built successfully with static linking");
}

/// Build libcodex with dynamic linking
fn build_libcodex_dynamic(nim_codex_dir: &PathBuf) {
    println!("cargo:warning=Building libcodex with dynamic linking...");

    // Get CODEX_LIB_PARAMS from environment if set
    let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

    let mut make_cmd = Command::new("make");
    make_cmd.args(&["-C", &nim_codex_dir.to_string_lossy(), "libcodex"]);

    // Add custom parameters if provided
    if !codex_params.is_empty() {
        make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
    }

    let status = make_cmd
        .status()
        .expect("Failed to execute make command. Make sure make is installed and in PATH.");

    if !status.success() {
        panic!("Failed to build libcodex with dynamic linking");
    }

    println!("cargo:warning=libcodex built successfully with dynamic linking");
}

/// Ensure libcodex is built (check if it exists)
fn ensure_libcodex(nim_codex_dir: &PathBuf, lib_dir: &PathBuf, linking_mode: LinkingMode) {
    // Check if libcodex already exists
    let lib_exists = match linking_mode {
        LinkingMode::Static => lib_dir.join("libcodex.a").exists(),
        LinkingMode::Dynamic => lib_dir.join("libcodex.so").exists(),
    };

    if lib_exists {
        return;
    }

    match linking_mode {
        LinkingMode::Static => build_libcodex_static(nim_codex_dir),
        LinkingMode::Dynamic => build_libcodex_dynamic(nim_codex_dir),
    }
}

/// Link static library and its dependencies
fn link_static_library(nim_codex_dir: &PathBuf, _lib_dir: &PathBuf) {
    // Set up all library search paths first
    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/vendor/libbacktrace-upstream/.libs")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-circom-compat/vendor/circom-compat-ffi/target/release")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-nat-traversal/vendor/libnatpmp-upstream")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/install/usr/lib")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("nimcache/release/libcodex/vendor_leopard")
            .display()
    );

    // Use a custom linker script to handle the grouping properly
    // This avoids issues with Rust's automatic -Bstatic/-Bdynamic insertion
    println!("cargo:rustc-link-arg=-Wl,--whole-archive");

    // Link against additional required static libraries FIRST
    println!("cargo:rustc-link-lib=static=backtrace");
    println!("cargo:rustc-link-lib=static=circom_compat_ffi");
    println!("cargo:rustc-link-lib=static=natpmp");
    println!("cargo:rustc-link-lib=static=miniupnpc");
    println!("cargo:rustc-link-lib=static=backtracenim");
    println!("cargo:rustc-link-lib=static=libleopard");

    // Link against libcodex LAST (it depends on all the above)
    println!("cargo:rustc-link-lib=static=codex");

    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");

    // Link against C++ standard library for libcodex C++ dependencies
    println!("cargo:rustc-link-lib=stdc++");

    // Link against OpenMP for leopard library
    println!("cargo:rustc-link-lib=dylib=gomp");

    // Link against Rust's built-in stack probe for wasmer
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("cargo:rustc-link-arg=-Wl,--defsym=__rust_probestack=0");

    // Provide dummy symbols for missing Nim runtime functions
    println!("cargo:rustc-link-arg=-Wl,--defsym=cmdCount=0");
    println!("cargo:rustc-link-arg=-Wl,--defsym=cmdLine=0");

    println!("cargo:warning=Using static libcodex");
}

/// Link dynamic library
fn link_dynamic_library(lib_dir: &PathBuf) {
    println!("cargo:rustc-link-lib=dylib=codex");

    // Add rpath so the library can be found without LD_LIBRARY_PATH
    let lib_dir_abs = std::fs::canonicalize(lib_dir).unwrap_or_else(|_| lib_dir.to_path_buf());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir_abs.display());

    println!("cargo:warning=Using dynamic libcodex");
}

fn main() {
    let linking_mode = determine_linking_mode();
    let nim_codex_dir = PathBuf::from("vendor/nim-codex");
    let lib_dir = nim_codex_dir.join("build");
    let include_dir = nim_codex_dir.join("nimcache/release/libcodex");

    ensure_submodules(&nim_codex_dir);

    match linking_mode {
        LinkingMode::Static => {
            println!("cargo:warning=Building with static linking...");
            ensure_libcodex(&nim_codex_dir, &lib_dir, LinkingMode::Static);
            link_static_library(&nim_codex_dir, &lib_dir);
        }
        LinkingMode::Dynamic => {
            println!("cargo:warning=Building with dynamic linking...");
            ensure_libcodex(&nim_codex_dir, &lib_dir, LinkingMode::Dynamic);
            link_dynamic_library(&lib_dir);
        }
    }

    // Tell cargo to look for libraries in the build directory
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    generate_bindings(&include_dir, &nim_codex_dir);
}

/// Generate Rust bindings from C headers
fn generate_bindings(include_dir: &PathBuf, nim_codex_dir: &PathBuf) {
    // Verify include directory exists
    if !include_dir.exists() {
        panic!(
            "Include directory not found at {}. Please ensure libcodex was built successfully.",
            include_dir.display()
        );
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/bridge.h")
        // Add include path for libcodex headers
        .clang_arg(format!("-I{}", include_dir.display()))
        // Add include path for Nim headers
        .clang_arg(format!(
            "-I{}",
            nim_codex_dir
                .join("vendor/nimbus-build-system/vendor/Nim/lib")
                .display()
        ))
        // Tell bindgen to generate Rust bindings for all C++ enums.
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        // Tell bindgen to generate blocking functions.
        .generate_block(true)
        // Tell bindgen to generate layout tests.
        .layout_tests(false)
        // Tell bindgen to allowlist these types.
        .allowlist_function("codex_.*")
        .allowlist_type("codex_.*")
        .allowlist_var("codex_.*")
        .allowlist_var("RET_.*")
        // Suppress the naming convention warning for the generated type
        .raw_line("#[allow(non_camel_case_types)]")
        // Add a type alias to fix the naming convention issue
        .raw_line("pub type CodexCallback = tyProc__crazOL9c5Gf8j9cqs2fd61EA;")
        // Don't add imports here as they're already imported in ffi/mod.rs
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Rerun build script if these files change
    println!("cargo:rerun-if-changed=src/bridge.h");
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("libcodex.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        nim_codex_dir.join("build/libcodex.so").display()
    );
}
