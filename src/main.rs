use llm_md_cleaner::clean_markdown;
use std::cell::RefCell;
use std::rc::Rc;

mod theme;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // 1. Load the configuration from theme.toml using the new path logic
    let config = theme::load();

    // 2. Set SLINT_STYLE BEFORE initializing the UI (skip if empty)
    if !config.slint_style.is_empty() {
        std::env::set_var("SLINT_STYLE", &config.slint_style);
    }

    // 3. Create the UI
    let ui = AppWindow::new()?;

    // 4. Apply initial loaded settings to the UI
    theme::apply_active_theme(&ui, &config);
    theme::apply_fonts(&ui, &config);

    // Wrap config in Rc<RefCell> so callbacks can mutate and save it
    let shared_config = Rc::new(RefCell::new(config));

    // --- Core Logic Callbacks ---

    let weak = ui.as_weak();
    ui.on_clean_requested(move || {
        let ui = weak.unwrap();

        let input = ui.get_input_text().to_string();
        let cleaned = clean_markdown(&input);

        let status = format!(
            "Cleaned {} input characters into {} output characters.",
            input.chars().count(),
            cleaned.chars().count()
        );

        ui.set_output_text(cleaned.into());
        ui.set_status_text(status.into());
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

    // --- Theme Callbacks ---

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_theme_moribund_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = cfg_clone.borrow_mut();

        cfg.active_theme = "moribund".to_string();
        theme::apply_active_theme(&ui, &cfg);
        theme::save(&cfg);
    });

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_theme_dark_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = cfg_clone.borrow_mut();

        cfg.active_theme = "dark".to_string();
        theme::apply_active_theme(&ui, &cfg);
        theme::save(&cfg);
    });

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_theme_light_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = cfg_clone.borrow_mut();

        cfg.active_theme = "light".to_string();
        theme::apply_active_theme(&ui, &cfg);
        theme::save(&cfg);
    });

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_theme_system_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = cfg_clone.borrow_mut();
        
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

    // --- Font Callbacks ---

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_editor_font_default_requested(move || {
        let ui = weak.unwrap();
        ui.set_editor_font_family("".into());
        ui.set_status_text("Editor font changed to default.".into());

        let mut cfg = cfg_clone.borrow_mut();
        cfg.editor_font = "".to_string();
        theme::save(&cfg);
    });

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_editor_font_monospace_requested(move || {
        let ui = weak.unwrap();
        ui.set_editor_font_family("monospace".into());
        ui.set_status_text("Editor font changed to monospace.".into());

        let mut cfg = cfg_clone.borrow_mut();
        cfg.editor_font = "monospace".to_string();
        theme::save(&cfg);
    });

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_editor_font_larger_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = cfg_clone.borrow_mut();

        cfg.editor_font_size += 1.0;
        ui.set_editor_font_size(cfg.editor_font_size);
        ui.set_status_text("Editor font size increased.".into());

        theme::save(&cfg);
    });

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();
    ui.on_editor_font_smaller_requested(move || {
        let ui = weak.unwrap();
        let mut cfg = cfg_clone.borrow_mut();

        if cfg.editor_font_size > 8.0 {
            cfg.editor_font_size -= 1.0;
        }

        ui.set_editor_font_size(cfg.editor_font_size);
        ui.set_status_text("Editor font size decreased.".into());

        theme::save(&cfg);
    });

    // --- Font Upload Callback ---

    let weak = ui.as_weak();
    let cfg_clone = shared_config.clone();

    ui.on_editor_font_upload_requested(move || {
        let ui = weak.unwrap();

        // 1. Open the File Dialog
        let Some(path) = rfd::FileDialog::new()
            .set_title("Choose editor font")
            .add_filter("Font files", &["ttf", "otf"])
            .pick_file()
        else {
            ui.set_status_text("Font upload canceled.".into());
            return;
        };

        let mut cfg = cfg_clone.borrow_mut();

        // 2. Process the import using the new logic in theme.rs
        match theme::import_editor_font(&path, &mut cfg) {
            Ok(family_name) => {
                // Register and apply the font immediately
                theme::apply_fonts(&ui, &cfg);
                theme::save(&cfg);

                ui.set_status_text(format!("Editor font imported: {family_name}").into());
            }
            Err(err) => {
                // Report specific errors (wrong extension, copy failure, etc.)
                ui.set_status_text(format!("Font import failed: {err}").into());
            }
        }
    });

    ui.run()
}
