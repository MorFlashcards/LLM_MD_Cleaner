use crate::AppWindow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use slint::fontique_08::fontique;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_active_theme")]
    pub active_theme: String,

    #[serde(default = "default_slint_style")]
    pub slint_style: String,

    // UI chrome font: headings, labels, buttons, status text, etc.
    // Keep this separate from the Markdown editor font.
    #[serde(default = "default_ui_font")]
    pub ui_font: String,

    #[serde(default = "default_ui_font_file")]
    pub ui_font_file: String,

    // Markdown editor font: messy input and clean Markdown output boxes.
    // This should stay monospace by default.
    #[serde(default = "default_editor_font")]
    pub editor_font: String,

    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: f32,

    #[serde(default)]
    pub editor_font_file: String,

    #[serde(default = "default_theme_map")]
    pub themes: BTreeMap<String, ThemePalette>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemePalette {
    pub app_background: String,
    pub toolbar_background: String,
    pub panel_background: String,
    pub border_color: String,
    pub divider_color: String,
    pub heading_color: String,
    pub body_text_color: String,
    pub muted_text_color: String,

    #[serde(default)]
    pub status_message: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            active_theme: default_active_theme(),
            slint_style: default_slint_style(),

            ui_font: default_ui_font(),
            ui_font_file: default_ui_font_file(),

            editor_font: default_editor_font(),
            editor_font_size: default_editor_font_size(),
            editor_font_file: String::new(),

            themes: default_theme_map(),
        }
    }
}

fn default_active_theme() -> String {
    "moribund".to_string()
}

fn default_slint_style() -> String {
    "native".to_string()
}

fn default_ui_font() -> String {
    "IM FELL English".to_string()
}

fn default_ui_font_file() -> String {
    "assets/fonts/IMFellEnglish-Regular.ttf".to_string()
}

fn default_editor_font() -> String {
    "monospace".to_string()
}

fn default_editor_font_size() -> f32 {
    12.0
}

fn default_theme_map() -> BTreeMap<String, ThemePalette> {
    let mut themes = BTreeMap::new();

    themes.insert(
        "moribund".to_string(),
        ThemePalette {
            app_background: "#181920".to_string(),
            toolbar_background: "#14151c".to_string(),
            panel_background: "#101117".to_string(),
            border_color: "#2f3140".to_string(),
            divider_color: "#3b4058".to_string(),
            heading_color: "#a77be8".to_string(),
            body_text_color: "#d7d4cf".to_string(),
            muted_text_color: "#6f7388".to_string(),
            status_message: "Theme changed to Moribund.".to_string(),
        },
    );

    themes.insert(
        "dark".to_string(),
        ThemePalette {
            app_background: "#1e1e1e".to_string(),
            toolbar_background: "#2a2a2a".to_string(),
            panel_background: "#161616".to_string(),
            border_color: "#505050".to_string(),
            divider_color: "#5f5f5f".to_string(),
            heading_color: "#e6e6e6".to_string(),
            body_text_color: "#c8c8c8".to_string(),
            muted_text_color: "#969696".to_string(),
            status_message: "Theme changed to Plain Dark.".to_string(),
        },
    );

    themes.insert(
        "light".to_string(),
        ThemePalette {
            app_background: "#f5f5f5".to_string(),
            toolbar_background: "#e8e8e8".to_string(),
            panel_background: "#ffffff".to_string(),
            border_color: "#bebebe".to_string(),
            divider_color: "#aaaaaa".to_string(),
            heading_color: "#232323".to_string(),
            body_text_color: "#414141".to_string(),
            muted_text_color: "#696969".to_string(),
            status_message: "Theme changed to Plain Light.".to_string(),
        },
    );

    themes
}

// --- Path Helpers ---

pub fn config_dir() -> PathBuf {
    ProjectDirs::from("org", "MoribundMurdoch", "LLM_MD_Cleaner")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn theme_path() -> PathBuf {
    config_dir().join("theme.toml")
}

pub fn fonts_dir() -> PathBuf {
    config_dir().join("fonts")
}

fn resolve_project_path(path: &str) -> PathBuf {
    let raw = PathBuf::from(path);

    if raw.is_absolute() {
        raw
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(raw)
    }
}

// --- Persistence ---

pub fn load() -> ThemeConfig {
    let user_theme = theme_path();

    if let Ok(contents) = fs::read_to_string(&user_theme) {
        toml::from_str(&contents).unwrap_or_default()
    } else if let Ok(contents) = fs::read_to_string("assets/config/theme.toml") {
        let config: ThemeConfig = toml::from_str(&contents).unwrap_or_default();
        save(&config);
        config
    } else if let Ok(contents) = fs::read_to_string("theme.toml") {
        let config: ThemeConfig = toml::from_str(&contents).unwrap_or_default();
        save(&config);
        config
    } else {
        let config = ThemeConfig::default();
        save(&config);
        config
    }
}

pub fn save(config: &ThemeConfig) {
    let _ = fs::create_dir_all(config_dir());

    if let Ok(toml_str) = toml::to_string_pretty(config) {
        let _ = fs::write(theme_path(), toml_str);
    }
}

// --- Theme and Font Application ---

pub fn apply_active_theme(ui: &AppWindow, config: &ThemeConfig) {
    apply_palette(ui, config, &config.active_theme);
}

pub fn apply_palette(ui: &AppWindow, config: &ThemeConfig, theme_name: &str) {
    let Some(theme) = config.themes.get(theme_name) else {
        ui.set_status_text(format!("Theme '{theme_name}' was not found in theme.toml.").into());
        return;
    };

    if let Some(color) = parse_hex_color(&theme.app_background) {
        ui.set_app_bg(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.toolbar_background) {
        ui.set_toolbar_bg(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.panel_background) {
        ui.set_panel_bg(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.border_color) {
        ui.set_theme_border(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.divider_color) {
        ui.set_theme_divider(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.heading_color) {
        ui.set_heading_fg(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.body_text_color) {
        ui.set_body_fg(color.into());
    }
    if let Some(color) = parse_hex_color(&theme.muted_text_color) {
        ui.set_muted_fg(color.into());
    }

    if theme.status_message.trim().is_empty() {
        ui.set_status_text(format!("Theme changed to {theme_name}.").into());
    } else {
        ui.set_status_text(theme.status_message.clone().into());
    }
}

pub fn apply_fonts(ui: &AppWindow, config: &ThemeConfig) {
    register_configured_font(ui, "UI", &config.ui_font_file);
    register_configured_font(ui, "editor", &config.editor_font_file);

    ui.set_ui_font_family(config.ui_font.clone().into());
    ui.set_editor_font_family(config.editor_font.clone().into());
    ui.set_editor_font_size(config.editor_font_size);
}

fn register_configured_font(ui: &AppWindow, label: &str, configured_path: &str) {
    if configured_path.trim().is_empty() {
        return;
    }

    let path = resolve_project_path(configured_path);

    if path.exists() {
        if let Err(err) = register_font_from_path_fontique(&path) {
            ui.set_status_text(format!("Could not load {label} font: {err}").into());
        }
    } else {
        ui.set_status_text(
            format!(
                "Configured {label} font file was not found: {}",
                path.display()
            )
            .into(),
        );
    }
}

// --- Font Import and Detection ---

pub fn import_editor_font(source_path: &Path, config: &mut ThemeConfig) -> Result<String, String> {
    let destination = copy_font_into_config_dir(source_path)?;

    register_font_from_path_fontique(&destination)
        .map_err(|err| format!("Slint could not load this font: {err}"))?;

    let family_name = detect_font_family(&destination).unwrap_or_else(|| {
        destination
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "Custom Font".to_string())
    });

    config.editor_font_file = destination.to_string_lossy().to_string();
    config.editor_font = family_name.clone();

    Ok(family_name)
}

#[allow(dead_code)]
pub fn import_ui_font(source_path: &Path, config: &mut ThemeConfig) -> Result<String, String> {
    let destination = copy_font_into_config_dir(source_path)?;

    register_font_from_path_fontique(&destination)
        .map_err(|err| format!("Slint could not load this font: {err}"))?;

    let family_name = detect_font_family(&destination).unwrap_or_else(|| {
        destination
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "Custom Font".to_string())
    });

    config.ui_font_file = destination.to_string_lossy().to_string();
    config.ui_font = family_name.clone();

    Ok(family_name)
}

fn copy_font_into_config_dir(source_path: &Path) -> Result<PathBuf, String> {
    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension != "ttf" && extension != "otf" {
        return Err("Please choose a .ttf or .otf font file.".to_string());
    }

    fs::create_dir_all(fonts_dir())
        .map_err(|err| format!("Could not create fonts folder: {err}"))?;

    let file_name = source_path
        .file_name()
        .ok_or_else(|| "Font file had no filename.".to_string())?;

    let destination = fonts_dir().join(file_name);

    fs::copy(source_path, &destination)
        .map_err(|err| format!("Could not copy font into config folder: {err}"))?;

    Ok(destination)
}

fn register_font_from_path_fontique(path: &Path) -> Result<(), String> {
    let font_data = fs::read(path)
        .map_err(|err| format!("Could not read font file {}: {err}", path.display()))?;

    let blob = fontique::Blob::new(Arc::new(font_data));
    let mut collection = slint::fontique_08::shared_collection();
    let registered_fonts = collection.register_fonts(blob, None);

    if registered_fonts.is_empty() {
        return Err(format!("No usable fonts found in {}", path.display()));
    }

    Ok(())
}

fn detect_font_family(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    let face = ttf_parser::Face::parse(&data, 0).ok()?;

    // Priority 1: Family Name (e.g., "IM FELL English")
    for name in face.names() {
        if name.name_id == ttf_parser::name_id::FAMILY {
            if let Some(value) = name.to_string() {
                return Some(value);
            }
        }
    }

    // Priority 2: Full Name (e.g., "IM FELL English Regular")
    for name in face.names() {
        if name.name_id == ttf_parser::name_id::FULL_NAME {
            if let Some(value) = name.to_string() {
                return Some(value);
            }
        }
    }

    None
}

fn parse_hex_color(input: &str) -> Option<slint::Color> {
    let hex = input.trim().trim_start_matches('#');

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(slint::Color::from_rgb_u8(r, g, b))
}
