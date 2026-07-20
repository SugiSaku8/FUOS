use anyhow::Result;

use crate::{gebco::GebcoLoader, heightmap::HeightMap};

pub struct HeightMapLoader;

impl HeightMapLoader {
    //
    // テスト地形
    //
    pub fn load_test() -> HeightMap {
        HeightMap::test()
    }

    //
    // GEBCO NetCDF
    //
    pub fn load_gebco(path: &str) -> Result<HeightMap> {
        GebcoLoader::load(path)
    }
}
