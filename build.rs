fn main() {
    println!("cargo:rerun-if-changed=ui/app.slint");
    println!("cargo:rerun-if-changed=assets/config/theme.toml");
    println!("cargo:rerun-if-changed=assets/fonts/IMFellEnglish-Regular.ttf");

    slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");
}
