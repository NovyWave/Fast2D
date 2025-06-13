use download_cef::{download_target_archive, extract_target_archive};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Use the local CEF version  
    let version = "137.0.17";
    let target = std::env::var("TARGET").unwrap();
    
    println!("Building CEF for target: {}", target);
    println!("Using CEF version: {}", version);
    
    // Download and extract CEF if needed
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let archive_path = download_target_archive(&target, version, &out_dir, true)
        .expect("Failed to download CEF archive");
    
    let cef_dir = extract_target_archive(&target, &archive_path, &out_dir, true)
        .expect("Failed to extract CEF archive");
    
    println!("CEF extracted to: {}", cef_dir.display());
    
    // Set CEF_PATH for the build
    println!("cargo:rustc-env=CEF_PATH={}", cef_dir.display());
    
    println!("CEF build configuration complete");
}