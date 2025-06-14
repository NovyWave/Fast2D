use download_cef::{download_target_archive, extract_target_archive};

// Macro for build.rs output using cargo:warning (the correct way to show messages)
macro_rules! build_info {
    ($($tokens: tt)*) => {
        println!("cargo:warning=[BUILD] {}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Use the local CEF version  
    let version = "137.0.17";
    let target = std::env::var("TARGET").unwrap();
    
    build_info!("🔧 Building CEF for target: {}", target);
    build_info!("📦 Using CEF version: {}", version);
    
    // Download CEF to local cef_downloads directory
    let cef_downloads_dir = std::path::Path::new("cef_downloads");
    std::fs::create_dir_all(&cef_downloads_dir).expect("Failed to create cef_downloads directory");
    
    build_info!("📁 CEF downloads directory: {}", cef_downloads_dir.display());
    
    let archive_path = match download_target_archive(&target, version, &cef_downloads_dir, true) {
        Ok(path) => {
            build_info!("✅ CEF archive downloaded to: {}", path.display());
            path
        }
        Err(e) => {
            build_info!("❌ Failed to download CEF archive: {}", e);
            build_info!("💡 This might be due to network issues or missing CEF binaries");
            build_info!("💡 Try running again or check your internet connection");
            build_info!("💡 Expected download size: ~100-200MB from cef-builds.spotifycdn.com");
            panic!("Failed to download CEF archive: {}", e);
        }
    };
    
    build_info!("📂 Extracting CEF archive (~1-2 minutes)...");
    let cef_dir = match extract_target_archive(&target, &archive_path, &cef_downloads_dir, true) {
        Ok(dir) => {
            build_info!("✅ CEF archive extracted to: {}", dir.display());
            dir
        }
        Err(e) => {
            build_info!("❌ Failed to extract CEF archive: {}", e);
            panic!("Failed to extract CEF archive: {}", e);
        }
    };
    
    // Set CEF_PATH for the build
    println!("cargo:rustc-env=CEF_PATH={}", cef_dir.display());
    
    build_info!("🎯 CEF build configuration complete!");
    build_info!("📊 Total estimated time: 3-5 minutes for download + extraction");
}