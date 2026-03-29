use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;

pub fn write_object(blob: &Vec<u8>) -> Result<String, String> {
    let hash_hex = Sha1::digest(&blob)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&blob)
        .map_err(|e| format!("failed to encode object file: {}", e))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("failed to compress object file: {}", e))?;

    let dir = format!(".minigit/objects/{}", &hash_hex[..2]);
    let file_path = format!("{}/{}", dir, &hash_hex[2..]);

    fs::create_dir_all(&dir).map_err(|e| format!("failed to create object dir: {}", e))?;
    fs::write(&file_path, compressed).map_err(|e| format!("failed to write object: {}", e))?;

    return Ok(hash_hex);
}

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
