use crate::grid::Grid2D;

/// 2次元フィールド
///
/// メモリレイアウト
///
/// data =
/// [
///   (0,0),(1,0),(2,0),
///   (0,1),(1,1),(2,1),
///   ...
/// ]
///
/// index = y * width + x
#[derive(Clone, Debug)]
pub struct Field2D<T> {
    grid: Grid2D,
    data: Vec<T>,
}

impl<T: Clone> Field2D<T> {
    /// 全要素を value で初期化
    pub fn new(grid: Grid2D, value: T) -> Self {
        let size = grid.width * grid.height;

        Self {
            grid,
            data: vec![value; size],
        }
    }

    /// Vecから生成
    pub fn from_vec(grid: Grid2D, data: Vec<T>) -> Self {
        assert_eq!(
            grid.width * grid.height,
            data.len(),
            "Field2D size mismatch"
        );

        Self { grid, data }
    }

    /// 全要素を同じ値で埋める
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }
}

impl<T> Field2D<T> {
    #[inline]
    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.grid.width || y >= self.grid.height {
            return None;
        }

        Some(y * self.grid.width + x)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.index(x, y).map(|i| &self.data[i])
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.index(x, y).map(move |i| &mut self.data[i])
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        if let Some(i) = self.index(x, y) {
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
    pub fn grid(&self) -> &Grid2D {
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
