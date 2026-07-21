use crate::grid::Grid3D;

/// 3次元フィールド
///
/// メモリレイアウト
///
/// data =
/// [
///   z = 0
///     y = 0
///       x = 0..width
///
///     y = 1
///       x = ...
///
///   z = 1
///     ...
/// ]
///
/// index = (z * height + y) * width + x
#[derive(Clone, Debug)]
pub struct Field3D<T> {
    grid: Grid3D,
    data: Vec<T>,
}

impl<T: Clone> Field3D<T> {
    /// 全要素を value で初期化
    pub fn new(grid: Grid3D, value: T) -> Self {
        Self {
            data: vec![value; grid.len()],
            grid,
        }
    }

    /// Vecから生成
    pub fn from_vec(grid: Grid3D, data: Vec<T>) -> Self {
        assert_eq!(grid.len(), data.len(), "Field3D size mismatch");

        Self { grid, data }
    }

    /// 全要素を同じ値で埋める
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }
}

impl<T> Field3D<T> {
    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= self.grid.width {
            return None;
        }

        if y >= self.grid.height {
            return None;
        }

        if z >= self.grid.depth {
            return None;
        }

        Some((z * self.grid.height + y) * self.grid.width + x)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        self.index(x, y, z).map(|i| &self.data[i])
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut T> {
        self.index(x, y, z).map(move |i| &mut self.data[i])
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: T) -> bool {
        if let Some(i) = self.index(x, y, z) {
            self.data[i] = value;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.grid.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.grid.height
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.grid.depth
    }

    #[inline]
    pub fn grid(&self) -> &Grid3D {
        &self.grid
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }
}
