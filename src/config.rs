use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn default_vendor() -> u16 {
    0x3151
}
fn default_product() -> u16 {
    0x502d
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
        ("ESC", 0),
        ("BACKTICK", 1),
        ("TAB", 2),
        ("CAPS", 3),
        ("L_SHIFT", 4),
        ("L_CTRL", 5),
        ("F1", 6),
        ("1", 7),
        ("Q", 8),
        ("A", 9),
        ("F2", 12),
        ("2", 13),
        ("W", 14),
        ("S", 15),
        ("Z", 16),
        ("L_SUPER", 17),
        ("F3", 18),
        ("3", 19),
        ("E", 20),
        ("D", 21),
        ("X", 22),
        ("L_ALT", 23),
        ("F4", 24),
        ("4", 25),
        ("R", 26),
        ("F", 27),
        ("C", 28),
        ("F5", 30),
        ("5", 31),
        ("T", 32),
        ("G", 33),
        ("V", 34),
        ("F6", 36),
        ("6", 37),
        ("Y", 38),
        ("H", 39),
        ("B", 40),
        ("F7", 42),
        ("SPACE", 41),
        ("7", 43),
        ("U", 44),
        ("J", 45),
        ("N", 46),
        ("F8", 48),
        ("8", 49),
        ("I", 50),
        ("K", 51),
        ("M", 52),
        ("F9", 54),
        ("9", 55),
        ("O", 56),
        ("L", 57),
        ("COMMA", 58),
        ("R_ALT", 59),
        ("F10", 60),
        ("0", 61),
        ("P", 62),
        ("COLON", 63),
        ("DOT", 64),
        ("FN", 65),
        ("F11", 66),
        ("MINUS", 67),
        ("L_BRACE", 68),
        ("QUOTE", 69),
        ("SLASH", 70),
        ("R_CTRL", 71),
        ("F12", 72),
        ("EQUAL", 73),
        ("R_BRACE", 74),
        ("R_SHIFT", 76),
        ("LEFT", 77),
        ("DELETE", 78),
        ("BACKSPACE", 79),
        ("PIPE", 80),
        ("ENTER", 81),
        ("UP", 82),
        ("DOWN", 83),
        ("INSERT", 84),
        ("HOME", 85),
        ("PAGE_UP", 86),
        ("PAGE_DOWN", 87),
        ("END", 88),
        ("RIGHT", 89),
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
