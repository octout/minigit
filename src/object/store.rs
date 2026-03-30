use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};

use super::GitObject;

pub fn save(obj: &impl GitObject) -> Result<String, String> {
    let body = obj.serialize_body();
    let header = format!("{} {}\0", obj.object_type(), body.len());
    let mut data = header.into_bytes();
    data.extend(body);
    save_bytes(&data)
}

pub fn save_bytes(data: &[u8]) -> Result<String, String> {
    let hash_hex = Sha1::digest(data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| format!("failed to encode object: {}", e))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("failed to compress object: {}", e))?;

    let dir = format!(".minigit/objects/{}", &hash_hex[..2]);
    let file_path = format!("{}/{}", dir, &hash_hex[2..]);

    fs::create_dir_all(&dir).map_err(|e| format!("failed to create object dir: {}", e))?;
    fs::write(&file_path, compressed).map_err(|e| format!("failed to write object: {}", e))?;

    Ok(hash_hex)
}

pub fn load(hash_hex: &str) -> Result<(String, Vec<u8>), String> {
    let file_path = format!(".minigit/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]);
    let compressed = fs::read(&file_path).map_err(|e| format!("failed to read object: {}", e))?;

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("failed to decompress object: {}", e))?;

    let null_pos = decompressed
        .iter()
        .position(|&b| b == 0)
        .ok_or("invalid object format")?;
    let header = std::str::from_utf8(&decompressed[..null_pos])
        .map_err(|e| format!("invalid object header: {}", e))?;
    let object_type = header
        .split(' ')
        .next()
        .ok_or("invalid object header")?
        .to_string();
    let body = decompressed[null_pos + 1..].to_vec();

    Ok((object_type, body))
}

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hash(obj: &impl GitObject) -> String {
    let body = obj.serialize_body();
    let header = format!("{} {}\0", obj.object_type(), body.len());
    let mut data = header.into_bytes();
    data.extend(body);
    Sha1::digest(&data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
