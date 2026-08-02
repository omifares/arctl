mod config;

use anyhow::{Context, Result, bail};
use clap::Parser;
use hidapi::HidApi;
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn calc_checksum(packet: &[u8]) -> u8 {
    let sum: u16 = packet[0..7].iter().map(|&x| x as u16).sum();
    (255 - (sum & 0xFF)) as u8
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        bail!("Formato hexadecimal inválido: {}", hex);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).context("Error on Red channel")?;
    let g = u8::from_str_radix(&hex[2..4], 16).context("Error on Green channel")?;
    let b = u8::from_str_radix(&hex[4..6], 16).context("Error on Blue channel")?;
    Ok((r, g, b))
}

#[derive(Parser, Debug)]
#[command(name = "akko-rgb", author, version, about = "Linux RGB controller for Akko TAC75", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "~/.config/akko/")]
    config_dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let home = env::var("HOME").context("$HOME environment variable not found")?;
    let config_dir_path = PathBuf::from(cli.config_dir.replace("~", &home));

    let (hw_config, colors_config) = config::load_configurations(&config_dir_path)?;

    let base_rgb = hex_to_rgb(&colors_config.colors.base)?;
    let mod_rgb = hex_to_rgb(&colors_config.colors.modifiers)?;

    let mut flat_payload = vec![0u8; 112 * 3];

    for (key_name, &idx) in &hw_config.key_map {
        let mut color = base_rgb;

        if hw_config.modifiers_list.contains(key_name) {
            color = mod_rgb;
        }

        if let Some(overrides) = &colors_config.overrides {
            if let Some(hex) = overrides.get(key_name) {
                if let Ok(override_color) = hex_to_rgb(hex) {
                    color = override_color;
                }
            }
        }

        let mem_idx = idx * 3;

        if mem_idx + 2 < flat_payload.len() {
            flat_payload[mem_idx] = color.0;
            flat_payload[mem_idx + 1] = color.1;
            flat_payload[mem_idx + 2] = color.2;
        } else {
            eprintln!(
                "[-] Warning: Index {} of key {} exceeds memory buffer!",
                idx, key_name
            );
        }
    }

    let api = HidApi::new().context("Failed to initialize HID API")?;

    let target_path = api
        .device_list()
        .find_map(|dev| {
            if dev.vendor_id() == u16::from(hw_config.device.vendor_id)
                && dev.product_id() == u16::from(hw_config.device.product_id)
            {
                let intf = dev.interface_number();
                if intf == 1 || intf == 2 || dev.usage_page() == 0xFF00 {
                    return Some(dev.path().to_owned());
                }
            }
            None
        })
        .context("[-] Keyboard not found! Check USB connection or udev rules.")?;

    let dev = api
        .open_path(&target_path)
        .context("Failed to open HID port")?;

    // Handshake
    let mut init_pkt = vec![0u8; 65];
    init_pkt[1..10].copy_from_slice(&[7, 13, 4, 4, 64, 0, 200, 200, 19]);
    dev.send_feature_report(&init_pkt)
        .context("Failed to send initialization (handshake)")?;
    thread::sleep(Duration::from_millis(20));

    for page in 0..6 {
        let start = page * 56;
        let end = start + 56;

        let mut pkt = vec![0u8; 65];
        pkt[1..8].copy_from_slice(&[12, 4, 255, page as u8, 56, 0, 0]);
        pkt[9..65].copy_from_slice(&flat_payload[start..end]);
        pkt[8] = calc_checksum(&pkt[1..65]);

        dev.send_feature_report(&pkt)?;
        thread::sleep(Duration::from_millis(10));
    }

    let mut end_pkt = vec![0u8; 65];
    end_pkt[1..9].copy_from_slice(&[12, 4, 255, 6, 42, 1, 0, 191]);
    dev.send_feature_report(&end_pkt)?;

    println!("[+] Colors applied successfully!");
    Ok(())
}
