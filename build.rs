use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    // Platform-specific optimizations
    match target_os.as_str() {
        "windows" => {
            // Windows-specific build settings
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=kernel32");
            
            // Enable Windows subsystem for release builds
            if env::var("PROFILE").unwrap() == "release" {
                println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
                println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
            }
        }
        "macos" => {
            // macOS-specific build settings
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=AppKit");
        }
        "linux" => {
            // Linux-specific build settings
            println!("cargo:rustc-link-lib=X11");
            println!("cargo:rustc-link-lib=Xrandr");
        }
        _ => {}
    }
    
    // Add version info
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("cargo:rustc-env=BUILD_TARGET={}", target_os);
    
    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}