use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,

    pub creator: String,

    pub version: String,
}
use crate::grid::GridFile;
use crate::package::SofPackage;
use crate::repository::Repository;

pub struct GridRepository<'a> {
    package: &'a SofPackage,
}

impl<'a> Repository<GridFile> for GridRepository<'a> {
    fn load(&self) -> anyhow::Result<GridFile> {
        todo!()
    }

    fn save(&self, grid: &GridFile) -> anyhow::Result<()> {
        todo!()
    }
}
