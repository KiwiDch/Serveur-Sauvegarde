use std::{io, path::Path};

use crate::{
    domain::entities::hash::{FileHash, FileHashError},
    driven::stockage::{RepoDeleteError, StockageHash},
};

pub enum RemoveError {
    InvalidData(FileHashError),
    StreamError(io::Error),
    RepoError(RepoDeleteError)
}

impl From<FileHashError> for RemoveError {
    fn from(value: FileHashError) -> Self {
        Self::InvalidData(value)
    }
}

impl From<io::Error> for RemoveError {
    fn from(value: io::Error) -> Self {
        Self::StreamError(value)
    }
}

pub fn delete_file_hash<T>(path: &Path, stockage: &T) -> Result<(), RemoveError> where T:StockageHash<FileHash> {
    if let Err(e) = stockage.delete(path.to_str().unwrap()) {
        return Err(RemoveError::RepoError(e));
    }
    Ok(())
}