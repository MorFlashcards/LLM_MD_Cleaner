mod app;
mod theme;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    app::run()
}
