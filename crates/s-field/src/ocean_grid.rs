use crate::Grid4D;

/// 駿河湾海洋計算格子生成
///
/// x,y,z,time の4次元格子。
pub struct OceanGrid;

impl OceanGrid {
    /// 駿河湾用Grid4D生成
    ///
    /// nx : 東西方向
    /// ny : 南北方向
    /// nz : 深度方向
    /// nt : 時間
    pub fn suruga_bay(nx: usize, ny: usize, nz: usize, nt: usize) -> Grid4D {
        Grid4D::new([nx, ny, nz, nt], [1000.0, 1000.0, 10.0, 3600.0])
    }
}
