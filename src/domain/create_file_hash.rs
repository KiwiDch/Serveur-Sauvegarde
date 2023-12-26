use super::utils;
use std::{io, path::Path};

use crate::{
    domain::entities::hash::{FileHash, FileHashError},
    driven::{self, stockage::RepoCreateError},
};

pub enum CreateError {
    InvalidData(FileHashError),
    StreamError(io::Error),
    RepoError(RepoCreateError)
}

impl From<FileHashError> for CreateError {
    fn from(value: FileHashError) -> Self {
        Self::InvalidData(value)
    }
}

impl From<io::Error> for CreateError {
    fn from(value: io::Error) -> Self {
        Self::StreamError(value)
    }
}

pub fn create_file_hash<T>(path: &Path, stockage: &T) -> Result<FileHash, CreateError>
where
    T: driven::stockage::StockageHash<FileHash>,
{
    let hash = utils::calculate_hash(path)?.to_string();
    let path = path.to_owned();
    let file = FileHash::new(path, hash)?;

    match stockage.insert(file) {
        Ok(file) => Ok(file),
        Err(e) => Err(CreateError::RepoError(e))
    }
}
