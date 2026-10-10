use anyhow::Result;

use crate::{gebco::GebcoLoader, heightmap::HeightMap};

pub struct HeightMapLoader;

impl HeightMapLoader {
    //
    // テスト地形
    //
    pub fn load_test() -> HeightMap {
        HeightMap::load_sof("/Users/sugisaku/Desktop/Developer/Suruga-Wan/suruga_bay.sof/grid.snc")
            .expect("REASON")
    }

    //
    // GEBCO NetCDF
    //
    pub fn load_gebco(path: &str) -> Result<HeightMap> {
        GebcoLoader::load(path)
    }
}
