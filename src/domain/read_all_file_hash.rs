use crate::driven::stockage::{RepoFindAllError, StockageHash};
use std::path::Path;

use super::entities::hash::FileHash;

#[derive(Debug)]
pub enum ReadAllError {
    Unknown,
    RepoError(RepoFindAllError),
}

pub fn read_all_file_hash<T>(path: &Path, stockage: &T) -> Result<Vec<FileHash>, ReadAllError>
where
    T: StockageHash<FileHash>,
{
    let hashes = stockage.read_all();
    if let Err(e) = hashes {
        return Err(ReadAllError::RepoError(e));
    }
    let hashes: Vec<FileHash> = hashes
        .unwrap()
        .into_iter()
        .filter(|e| e.path().value().starts_with(&path))
        .collect();
    
    Ok(hashes)
}
