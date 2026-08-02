use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn default_vendor() -> u16 {
    0x3151
}
fn default_product() -> u16 {
    0x4015
}

fn default_modifiers() -> Vec<String> {
    vec![
        "L_CTRL".into(),
        "R_CTRL".into(),
        "L_SUPER".into(),
        "ALT".into(),
        "L_ALT".into(),
        "FN".into(),
        "L_SHIFT".into(),
        "R_SHIFT".into(),
        "CAPS".into(),
        "BACKSPACE".into(),
        "ENTER".into(),
        "TAB".into(),
        "SPACE".into(),
        "ESC".into(),
        "UP".into(),
        "DOWN".into(),
        "LEFT".into(),
        "RIGHT".into(),
        "DEL".into(),
        "INSERT".into(),
        "HOME".into(),
        "END".into(),
        "PAGE_UP".into(),
        "PAGE_DOWN".into(),
    ]
}

fn default_key_map() -> HashMap<String, usize> {
    let mut map = HashMap::new();
    let keys = [
        ("ESC", 2),
        ("TAB", 3),
        ("CAPS", 5),
        ("L_SUPER", 7),
        ("1", 8),
        ("Q", 9),
        ("A", 10),
        ("L_CTRL", 11),
        ("2", 14),
        ("W", 15),
        ("Z", 17),
        ("F3", 20),
        ("E", 22),
        ("L_ALT", 24),
        ("C", 29),
        ("F5", 33),
        ("V", 35),
        ("F6", 37),
        ("6", 48),
        ("Y", 39),
        ("H", 40),
        ("B", 41),
        ("SPACE", 42),
        ("F7", 43),
        ("7", 44),
        ("U", 45),
        ("J", 46),
        ("N", 47),
        ("8", 50),
        ("I", 51),
        ("K", 52),
        ("M", 53),
        ("F8", 54),
        ("F9", 76),
        ("9", 56),
        ("O", 57),
        ("L", 58),
        ("COMMA", 59),
        ("ALT", 60),
        ("F10", 61),
        ("0", 62),
        ("P", 63),
        ("SEMICOLON", 64),
        ("PERIOD", 65),
        ("FN", 66),
        ("F11", 67),
        ("MINUS", 68),
        ("L_BRACKET", 69),
        ("QUOTE", 70),
        ("SLASH", 71),
        ("R_CTRL", 72),
        ("F12", 73),
        ("EQUAL", 74),
        ("R_BRACKET", 75),
        ("L_SHIFT", 77),
        ("LEFT", 78),
        ("DEL", 79),
        ("BACKSPACE", 80),
        ("BACKSLASH", 81),
        ("ENTER", 82),
        ("UP", 83),
        ("DOWN", 84),
        ("INSERT", 85),
        ("HOME", 86),
        ("PAGE_UP", 87),
        ("PAGE_DOWN", 88),
        ("END", 89),
        ("RIGHT", 90),
        ("R_SHIFT", 91),
    ];
    for (k, v) in keys {
        map.insert(k.to_string(), v);
    }
    map
}

#[derive(Deserialize)]
pub struct HardwareConfig {
    #[serde(default)]
    pub device: Device,
    #[serde(default = "default_modifiers")]
    pub modifiers_list: Vec<String>,
    #[serde(default = "default_key_map")]
    pub key_map: HashMap<String, usize>,
}

#[derive(Deserialize, Default)]
pub struct Device {
    #[serde(default = "default_vendor")]
    pub vendor_id: u16,
    #[serde(default = "default_product")]
    pub product_id: u16,
}

#[derive(Deserialize, Default)]
pub struct ColorsConfig {
    #[serde(default)]
    pub colors: Colors,
    pub overrides: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct Colors {
    #[serde(default = "default_base_color")]
    pub base: String,
    #[serde(default = "default_mod_color")]
    pub modifiers: String,
}

fn default_base_color() -> String {
    "#ffffff".into()
}
fn default_mod_color() -> String {
    "#ff0000".into()
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            base: default_base_color(),
            modifiers: default_mod_color(),
        }
    }
}

pub fn load_configurations(config_dir: &Path) -> Result<(HardwareConfig, ColorsConfig)> {
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .with_context(|| format!("Fail to create directory: {:?}", config_dir))?;
    }

    let hw_path = config_dir.join("config.toml");
    let colors_path = config_dir.join("colors.toml");

    let hw_config = if hw_path.exists() {
        let content =
            fs::read_to_string(&hw_path).with_context(|| format!("Fail to read {:?}", hw_path))?;
        toml::from_str(&content).context("Fail to parse config.toml (invalid syntax)")?
    } else {
        let default_hw = HardwareConfig {
            device: Device::default(),
            modifiers_list: default_modifiers(),
            key_map: default_key_map(),
        };
        default_hw
    };

    let colors_config = if colors_path.exists() {
        let content = fs::read_to_string(&colors_path)
            .with_context(|| format!("Fail to read {:?}", colors_path))?;
        toml::from_str(&content).context("Fail to parse colors.toml (invalid syntax)")?
    } else {
        ColorsConfig::default()
    };

    Ok((hw_config, colors_config))
}
