use hidapi::HidApi;
use std::env;
use std::thread;
use std::time::Duration;

const VENDOR_ID: u16 = 0x3151;
const PRODUCT_ID: u16 = 0x502d;

fn calc_checksum(packet: &[u8]) -> u8 {
    let sum: u16 = packet[0..7].iter().map(|&x| x as u16).sum();
    (255 - (sum & 0xFF)) as u8
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), &'static str> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Formato hexadecimal inválido");
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Erro no Red")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Erro no Green")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Erro no Blue")?;
    Ok((r, g, b))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Uso: akko_wallust <#HEXCOLOR>");
        return Ok(());
    }

    let (r, g, b) = hex_to_rgb(&args[1])?;

    let api = HidApi::new()?;

    let target_path = api.device_list().find_map(|dev| {
        if dev.vendor_id() == VENDOR_ID && dev.product_id() == PRODUCT_ID {
            if dev.interface_number() == 1
                || dev.interface_number() == 2
                || dev.usage_page() == 0xFF00
            {
                return Some(dev.path().to_owned());
            }
        }
        None
    });

    let path = match target_path {
        Some(p) => p,
        None => {
            println!("[-] Teclado não encontrado!");
            return Ok(());
        }
    };

    let dev = api.open_path(&path)?;

    let mut flat_payload = vec![0u8; 112 * 3];
    for chunk in flat_payload.chunks_exact_mut(3) {
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
    }

    // O send_feature_report no Rust hidapi exige que o primeiro byte seja o Report ID (0x00).
    // Então os pacotes têm 65 bytes (1 byte ID + 64 bytes de payload).

    let mut init_pkt = vec![0u8; 65];
    init_pkt[1..10].copy_from_slice(&[7, 13, 4, 4, 64, 0, 200, 200, 19]);
    dev.send_feature_report(&init_pkt)?;
    thread::sleep(Duration::from_millis(20));

    for page in 0..6 {
        let start = page * 56;
        let end = start + 56;

        let mut pkt = vec![0u8; 65];
        pkt[1..8].copy_from_slice(&[12, 4, 255, page as u8, 56, 0, 0]);
        pkt[9..65].copy_from_slice(&flat_payload[start..end]);

        // tem que pula o ReportID
        pkt[8] = calc_checksum(&pkt[1..65]);

        dev.send_feature_report(&pkt)?;
        thread::sleep(Duration::from_millis(10));
    }

    let mut end_pkt = vec![0u8; 65];
    end_pkt[1..9].copy_from_slice(&[12, 4, 255, 6, 42, 1, 0, 191]);
    dev.send_feature_report(&end_pkt)?;

    println!("[+] Ok: Akko TAC75 pintado de R:{} G:{} B:{}", r, g, b);

    Ok(())
}
