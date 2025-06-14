// Build script for Servo example
//
// Handles platform-specific compilation and linking requirements for Servo

fn main() {
    // Platform-specific configuration
    #[cfg(target_os = "linux")]
    {
        // Link required system libraries for Servo on Linux
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xrandr");
        println!("cargo:rustc-link-lib=xcb");
        println!("cargo:rustc-link-lib=GL");
        println!("cargo:rustc-link-lib=EGL");
        
        // Add library search paths
        println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
        println!("cargo:rustc-link-search=/usr/lib");
    }

    #[cfg(target_os = "macos")]
    {
        // Link required frameworks for Servo on macOS
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=OpenGL");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=IOKit");
    }

    #[cfg(target_os = "windows")]
    {
        // Link required libraries for Servo on Windows
        println!("cargo:rustc-link-lib=opengl32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("cargo:rustc-link-lib=user32");
    }

    println!("cargo:rerun-if-changed=build.rs");
}