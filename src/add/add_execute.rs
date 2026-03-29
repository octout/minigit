use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;

use crate::common::index_readed;

pub fn add_file(path: &str) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| format!("failed to read file: {}", e))?;
    let header = format!("blob {}\0", content.len());
    let mut blob = header.into_bytes();
    blob.extend(&content);

    let hash_hex = Sha1::digest(&blob)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&blob)
        .map_err(|e| format!("failed to encode blob file: {}", e))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("failed to compress blob file: {}", e))?;

    let dir = format!(".minigit/objects/{}", &hash_hex[..2]);
    let file_path = format!("{}/{}", dir, &hash_hex[2..]);

    fs::create_dir_all(&dir).map_err(|e| format!("failed to create object dir: {}", e))?;
    fs::write(&file_path, compressed).map_err(|e| format!("failed to write hash object: {}", e))?;

    return Ok(hash_hex);
}

pub fn add_index(path: &str, hash_hex: &String) -> Result<(), String> {
    let mut index_vec = match index_readed::read_index() {
        Ok(vec) => vec,
        Err(_) => Vec::new(),
    };
    match index_vec.iter_mut().find(|idx| idx.path == path) {
        Some(index) => {
            index.hex = hash_hex.clone();
        },
        None => {
            index_vec.push(index_readed::IndexReaded::new(path, hash_hex, "create"));
        }
    }
    index_readed::write_index(&index_vec).map_err(|e| format!("failed to write index: {}", e))?;

    return Ok(());
}
