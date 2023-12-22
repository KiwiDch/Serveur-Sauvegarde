use std::path::PathBuf;

use crate::domain::entities::hash::{self, FileHash};

use super::stockage::{self, RepoCreateError, RepoDeleteError, RepoFindAllError};

struct FileHashSqlite {
    path: String,
    hash: String,
}

impl From<FileHash> for FileHashSqlite {
    fn from(value: FileHash) -> Self {
        FileHashSqlite {
            path: value.path().value().to_str().unwrap().to_string(),
            hash: value.hash().value().to_string(),
        }
    }
}

impl TryInto<FileHash> for FileHashSqlite {
    type Error = hash::FileHashError;

    fn try_into(self) -> Result<FileHash, Self::Error> {
        FileHash::new(PathBuf::from(self.path), self.hash)
    }
}

pub struct SqliteStockage {
    connection: sqlite::Connection,
}

impl SqliteStockage {
    pub fn new(chemin: &str) -> Self {
        let c = sqlite::open(chemin).unwrap();
        c.execute(
            "create table if not exists file_hash (
                 path text primary key,
                 hash text not null
             )",
        )
        .unwrap();
        SqliteStockage { connection: c }
    }
}

impl stockage::StockageHash<FileHash> for SqliteStockage {
    fn insert(&self, data: FileHash) -> Result<FileHash, RepoCreateError> {
        let value: FileHashSqlite = data.clone().into();

        if value.hash.is_empty() {
            return Err(RepoCreateError::InvalidData("empty hash".to_string()));
        }

        if value.path.is_empty() {
            return Err(RepoCreateError::InvalidData("path is empty".to_string()));
        }

        let q = "INSERT OR REPLACE INTO file_hash VALUES (?,?)";
        let mut statement = self.connection.prepare(q).unwrap();

        statement
            .bind(&[(1, &value.path[..]), (2, &value.hash[..])][..])
            .unwrap();
        statement.next().unwrap();
        Ok(data)
    }
    fn delete(&self, id: &str) -> Result<(), RepoDeleteError> {
        if id.is_empty() {
            return Err(RepoDeleteError::InvalidData("empty id".to_string()));
        }

        let q = "DELETE FROM file_hash WHERE path=?";
        let mut statement = self.connection.prepare(q).unwrap();
        statement.bind((1, id)).unwrap();
        statement.next().unwrap();
        Ok(())
    }
    fn read_all(&self) -> Result<Vec<FileHash>, RepoFindAllError> {
        let q = "SELECT * FROM file_hash";
        let mut v: Vec<FileHash> = Vec::new();
        let _ = self.connection.iterate(q, |paires| {
            let &[(_,Some(path)),(_,Some(hash))] = paires else {
                panic!("Erreur dans la bd");
            };

            v.push(FileHash::new(PathBuf::from(path), hash.to_string()).unwrap());

            true
        });

        Ok(v)
    }
}
