use camino::Utf8PathBuf;

pub struct SofPackage {
    root: Utf8PathBuf,
}
impl SofPackage {
    pub fn open<P: Into<Utf8PathBuf>>(path: P) -> anyhow::Result<Self> {
        Ok(Self { root: path.into() })
    }
    pub fn grid_path(&self) -> Utf8PathBuf {
        self.root.join("grid.snc")
    }

    pub fn boundary_path(&self) -> Utf8PathBuf {
        self.root.join("boundary.snc")
    }

    pub fn restart_path(&self) -> Utf8PathBuf {
        self.root.join("restart.snc")
    }
    pub fn grid(&self) -> GridRepository {
        GridRepository { package: self }
    }
}
