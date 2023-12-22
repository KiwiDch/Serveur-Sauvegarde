use std::{path::{Path, PathBuf}, io::{self, Read}, fs::File, collections::HashMap};
use sha2::{Digest,Sha256};
use walkdir::WalkDir;

pub fn calculate_hash(file_path: &Path) -> io::Result<String> {

    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn calculate_hash_recursive(path: &Path) -> HashMap<PathBuf, String> {
    let mut hashes: HashMap<PathBuf, String> = HashMap::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(hash) = calculate_hash(entry.path()) {
                hashes.insert(entry.path().to_owned(), hash.to_string());
            }
        } else if entry.path() != path {
            let h = calculate_hash_recursive(entry.path());
            for (key, value) in h.into_iter() {
                hashes.insert(key, value);
            }
        }
    }

    hashes
}