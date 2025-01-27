//! Build script for the slint app

/// Use `slint_build` to compile the slint view
fn main() {
    slint_build::compile("view/app.slint").expect("Slint build failed");
}
