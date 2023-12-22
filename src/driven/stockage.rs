use crate::domain::entities::Entity;

#[derive(Debug)]
pub enum RepoCreateError {
    InvalidData(String),
    Unknown(String)
}

#[derive(Debug)]
pub enum RepoFindAllError {
    Unknown(String)
}


#[derive(Debug)]
pub enum RepoDeleteError {
    NotFound,
    InvalidData(String),
    Unknown(String)
}

//#[async_trait]
pub trait StockageHash<T> where T: Entity {
    fn insert(&self, value: T) -> Result<T, RepoCreateError>;
    fn delete(&self, id: &str) -> Result<(), RepoDeleteError>;
    fn read_all(&self) -> Result<Vec<T>, RepoFindAllError>;
}