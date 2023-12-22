use std::path::{self, PathBuf};

use serde::{Deserialize, Serialize};

use super::Entity;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(String);

impl Hash {
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum HashError {
    EmptyHash,
}

impl TryFrom<String> for Hash {
    type Error = HashError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(HashError::EmptyHash);
        }

        Ok(Hash(value))
    }
}

#[derive(Debug)]
pub enum PathError {
    DoNotExist,
}

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, Deserialize)]
pub struct Path(PathBuf);

impl Path {
    pub fn value(&self) -> &path::Path {
        &(*self.0)
    }
}

impl TryFrom<PathBuf> for Path {
    type Error = PathError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        println!("{:?}",value);
        if !value.exists() {
            return Err(PathError::DoNotExist);
        }
        Ok(Path(value))
    }
}

#[derive(Debug, Clone)]
pub struct FileHash {
    path: Path,
    hash: Hash,
}

impl Entity for FileHash {}

impl FileHash {
    pub fn new(path: PathBuf, hash: String) -> Result<Self, FileHashError> {
        let path: Path = path.try_into()?;
        let hash: Hash = hash.try_into()?;

        Ok(FileHash { path, hash })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl Into<(Path, Hash)> for FileHash {
    fn into(self) -> (Path, Hash) {
        (self.path, self.hash)
    }
}

#[derive(Debug)]
pub enum FileHashError {
    HashError(HashError),
    PathError(PathError),
}

impl From<HashError> for FileHashError {
    fn from(value: HashError) -> Self {
        Self::HashError(value)
    }
}

impl From<PathError> for FileHashError {
    fn from(value: PathError) -> Self {
        Self::PathError(value)
    }
}
