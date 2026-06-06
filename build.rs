fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        // Génère l'icône programmatiquement et l'embed dans l'exe
        let ico_path = generate_ico();

        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "Dictum");
        res.set("FileDescription", "Dictée vocale locale — Whisper AI");
        res.set("LegalCopyright", "painteau");
        res.set_icon(&ico_path);
        if let Err(e) = res.compile() {
            eprintln!("winresource warning: {e}");
        }
    }
}

/// Génère un fichier .ico 32x32 (cercle bleu acier) dans OUT_DIR.
/// Retourne le chemin vers le fichier généré.
fn generate_ico() -> String {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let ico_path = format!("{out_dir}/dictum.ico");

    let size = 32u32;
    let mut pixels_bgra = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;
            if dist < 13.0 {
                pixels_bgra[idx]     = 180; // B
                pixels_bgra[idx + 1] = 130; // G
                pixels_bgra[idx + 2] = 70;  // R
                pixels_bgra[idx + 3] = 255; // A
            } else if dist < 15.0 {
                pixels_bgra[idx]     = 255;
                pixels_bgra[idx + 1] = 255;
                pixels_bgra[idx + 2] = 255;
                pixels_bgra[idx + 3] = 180;
            }
        }
    }

    // BITMAPINFOHEADER (40 bytes)
    let mut bmp: Vec<u8> = Vec::new();
    let height_doubled = size * 2; // ICO convention : hauteur doublée
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(size as i32).to_le_bytes());
    bmp.extend_from_slice(&(height_doubled as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());  // planes
    bmp.extend_from_slice(&32u16.to_le_bytes()); // bit count
    bmp.extend_from_slice(&0u32.to_le_bytes());  // compression BI_RGB
    bmp.extend_from_slice(&0u32.to_le_bytes());  // image size (0 = auto)
    bmp.extend_from_slice(&0i32.to_le_bytes());  // x pels/meter
    bmp.extend_from_slice(&0i32.to_le_bytes());  // y pels/meter
    bmp.extend_from_slice(&0u32.to_le_bytes());  // colors used
    bmp.extend_from_slice(&0u32.to_le_bytes());  // colors important

    // XOR mask — pixels en ordre bottom-up
    for y in (0..size).rev() {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            bmp.push(pixels_bgra[idx]);
            bmp.push(pixels_bgra[idx + 1]);
            bmp.push(pixels_bgra[idx + 2]);
            bmp.push(pixels_bgra[idx + 3]);
        }
    }

    // AND mask (1 bit/pixel, rows padded to 4 bytes) — tout opaque
    let and_row = ((size + 31) / 32 * 4) as usize;
    bmp.extend(vec![0u8; and_row * size as usize]);

    // En-tête ICO
    let data_offset: u32 = 6 + 16;
    let mut ico: Vec<u8> = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type = icon
    ico.extend_from_slice(&1u16.to_le_bytes()); // count

    // ICONDIRENTRY
    ico.push(size as u8);
    ico.push(size as u8);
    ico.push(0u8);
    ico.push(0u8);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&(bmp.len() as u32).to_le_bytes());
    ico.extend_from_slice(&data_offset.to_le_bytes());
    ico.extend_from_slice(&bmp);

    std::fs::write(&ico_path, &ico).expect("Impossible d'écrire dictum.ico");
    ico_path
}
