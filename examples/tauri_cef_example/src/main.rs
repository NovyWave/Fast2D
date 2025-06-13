// This is a placeholder for the workspace Cargo.toml
// The actual applications are in:
// - src-cef/src/main.rs (CEF desktop app)
// - frontend/src/main.rs (MoonZoon frontend) 
// - backend/src/main.rs (MoonZoon backend)

fn main() {
    println!("This is a workspace package. Run the actual applications:");
    println!("  makers cef_dev  # Start CEF app + MoonZoon server");
    println!("  makers cef      # Start CEF app only");
    println!("  makers mzoon start  # Start MoonZoon server only");
}