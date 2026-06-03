use crate::{theme, AppWindow};
use llm_md_cleaner::clean_markdown;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

type SharedConfig = Rc<RefCell<theme::ThemeConfig>>;

pub fn run() -> Result<(), slint::PlatformError> {
    let config = theme::load();

    if !config.slint_style.is_empty() {
        std::env::set_var("SLINT_STYLE", &config.slint_style);
    }

    let ui = AppWindow::new()?;
    theme::apply_active_theme(&ui, &config);
    theme::apply_fonts(&ui, &config);

    let shared_config = Rc::new(RefCell::new(config));

    wire_core_callbacks(&ui);
    wire_theme_callbacks(&ui, shared_config.clone());
    wire_font_callbacks(&ui, shared_config.clone());
    wire_font_upload_callback(&ui, shared_config);

    ui.run()
}

fn wire_core_callbacks(ui: &AppWindow) {
    let weak = ui.as_weak();
    ui.on_clean_requested(move || {
        let ui = weak.unwrap();
        let input = ui.get_input_text().to_string();
        let cleaned = clean_markdown(&input);

        ui.set_status_text(
            format!(
                "Cleaned {} input characters into {} output characters.",
                input.chars().count(),
                cleaned.chars().count()
            )
            .into(),
        );
        ui.set_output_text(cleaned.into());
    });

    let weak = ui.as_weak();
    ui.on_copy_requested(move || {
        let ui = weak.unwrap();
        let output = ui.get_output_text().to_string();

        if output.trim().is_empty() {
            ui.set_status_text("Nothing to copy yet.".into());
            return;
        }

        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(output)) {
            Ok(_) => ui.set_status_text("Copied cleaned Markdown to clipboard.".into()),
            Err(err) => ui.set_status_text(format!("Clipboard error: {err}").into()),
        }
    });
}

fn wire_theme_callbacks(ui: &AppWindow, config: SharedConfig) {
    let weak = ui.as_weak();
    let cfg = config.clone();
    ui.on_theme_moribund_requested(move || set_active_theme(&weak.unwrap(), &cfg, "moribund"));

    let weak = ui.as_weak();
    let cfg = config.clone();
    ui.on_theme_dark_requested(move || set_active_theme(&weak.unwrap(), &cfg, "dark"));

    let weak = ui.as_weak();
    let cfg = config.clone();
    ui.on_theme_light_requested(move || set_active_theme(&weak.unwrap(), &cfg, "light"));

    let weak = ui.as_weak();
    ui.on_theme_system_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = config.borrow_mut();

        ui.set_ui_font_family("".into());
        ui.set_editor_font_family("".into());
        ui.set_status_text(
            "System / Native mode selected. Restart with SLINT_STYLE=native for deeper platform styling."
                .into(),
        );

        cfg.active_theme = "system".to_string();
        cfg.slint_style = "native".to_string();
        theme::save(&cfg);
    });
}

fn set_active_theme(ui: &AppWindow, config: &SharedConfig, theme_name: &str) {
    let mut cfg = config.borrow_mut();
    cfg.active_theme = theme_name.to_string();
    theme::apply_active_theme(ui, &cfg);
    theme::save(&cfg);
}

fn wire_font_callbacks(ui: &AppWindow, config: SharedConfig) {
    let weak = ui.as_weak();
    let cfg = config.clone();
    ui.on_editor_font_default_requested(move || {
        set_editor_font(&weak.unwrap(), &cfg, "", "default")
    });

    let weak = ui.as_weak();
    let cfg = config.clone();
    ui.on_editor_font_monospace_requested(move || {
        set_editor_font(&weak.unwrap(), &cfg, "monospace", "monospace")
    });

    let weak = ui.as_weak();
    let cfg = config.clone();
    ui.on_editor_font_larger_requested(move || adjust_editor_font_size(&weak.unwrap(), &cfg, 1.0));

    let weak = ui.as_weak();
    ui.on_editor_font_smaller_requested(move || {
        adjust_editor_font_size(&weak.unwrap(), &config, -1.0)
    });
}

fn set_editor_font(ui: &AppWindow, config: &SharedConfig, family: &str, label: &str) {
    ui.set_editor_font_family(family.into());
    ui.set_status_text(format!("Editor font changed to {label}.").into());

    let mut cfg = config.borrow_mut();
    cfg.editor_font = family.to_string();
    theme::save(&cfg);
}

fn adjust_editor_font_size(ui: &AppWindow, config: &SharedConfig, delta: f32) {
    let mut cfg = config.borrow_mut();
    cfg.editor_font_size = (cfg.editor_font_size + delta).max(8.0);

    ui.set_editor_font_size(cfg.editor_font_size);

    if delta.is_sign_positive() {
        ui.set_status_text("Editor font size increased.".into());
    } else {
        ui.set_status_text("Editor font size decreased.".into());
    }

    theme::save(&cfg);
}

fn wire_font_upload_callback(ui: &AppWindow, config: SharedConfig) {
    let weak = ui.as_weak();

    ui.on_editor_font_upload_requested(move || {
        let ui = weak.unwrap();

        let Some(path) = rfd::FileDialog::new()
            .set_title("Choose editor font")
            .add_filter("Font files", &["ttf", "otf"])
            .pick_file()
        else {
            ui.set_status_text("Font upload canceled.".into());
            return;
        };

        let mut cfg = config.borrow_mut();

        match theme::import_editor_font(&path, &mut cfg) {
            Ok(family_name) => {
                theme::apply_fonts(&ui, &cfg);
                theme::save(&cfg);
                ui.set_status_text(format!("Editor font imported: {family_name}").into());
            }
            Err(err) => {
                ui.set_status_text(format!("Font import failed: {err}").into());
            }
        }
    });
}
