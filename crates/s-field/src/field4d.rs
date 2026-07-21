use crate::grid::Grid4D;

/// 4次元フィールド
///
/// インデックス順
///
/// (time, depth, y, x)
///
/// index =
/// (((t * depth + z) * height + y) * width + x)
#[derive(Clone, Debug)]
pub struct Field4D<T> {
    grid: Grid4D,
    data: Vec<T>,
}

impl<T: Clone> Field4D<T> {
    pub fn new(grid: Grid4D, value: T) -> Self {
        Self {
            data: vec![value; grid.len()],
            grid,
        }
    }

    pub fn from_vec(grid: Grid4D, data: Vec<T>) -> Self {
        assert_eq!(grid.len(), data.len(), "Field4D size mismatch");

        Self { grid, data }
    }

    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }
}

impl<T> Field4D<T> {
    #[inline]
    fn index(&self, t: usize, z: usize, y: usize, x: usize) -> Option<usize> {
        if x >= self.grid.width {
            return None;
        }

        if y >= self.grid.height {
            return None;
        }

        if z >= self.grid.depth {
            return None;
        }

        if t >= self.grid.time {
            return None;
        }

        Some((((t * self.grid.depth + z) * self.grid.height + y) * self.grid.width) + x)
    }

    #[inline]
    pub fn get(&self, t: usize, z: usize, y: usize, x: usize) -> Option<&T> {
        self.index(t, z, y, x).map(|i| &self.data[i])
    }

    #[inline]
    pub fn get_mut(&mut self, t: usize, z: usize, y: usize, x: usize) -> Option<&mut T> {
        self.index(t, z, y, x).map(move |i| &mut self.data[i])
    }

    #[inline]
    pub fn set(&mut self, t: usize, z: usize, y: usize, x: usize, value: T) -> bool {
        if let Some(i) = self.index(t, z, y, x) {
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
    pub fn time(&self) -> usize {
        self.grid.time
    }

    #[inline]
    pub fn grid(&self) -> &Grid4D {
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
