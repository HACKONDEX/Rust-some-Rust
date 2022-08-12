#![forbid(unsafe_code)]

#[derive(Clone, PartialEq, Eq)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    grid: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut obj = Self {
            rows,
            cols,
            grid: Vec::new(),
        };
        obj.grid.resize(rows * cols, T::default());
        obj
    }

    pub fn from_slice(grid: &[T], rows: usize, cols: usize) -> Self {
        let mut obj = Self {
            rows,
            cols,
            grid: Vec::new(),
        };
        obj.grid.extend_from_slice(grid);
        obj
    }

    pub fn size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn get(&self, row: usize, col: usize) -> &T {
        &self.grid[row * self.cols + col]
    }

    pub fn set(&mut self, value: T, row: usize, col: usize) {
        self.grid[row * self.cols + col] = value;
    }

    pub fn neighbours(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbours: Vec<(usize, usize)> = Vec::new();
        let positions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for pos in positions {
            let i: i32 = pos.0 + row as i32;
            let j: i32 = pos.1 + col as i32;
            if self.is_valid(i, j) {
                neighbours.push((i as usize, j as usize));
            }
        }
        neighbours
    }

    pub fn is_valid(&self, row: i32, col: i32) -> bool {
        row >= 0 && row < self.rows as i32 && col >= 0 && col < self.cols as i32
    }

    pub fn get_grid(&self) -> &Vec<T> {
        &self.grid
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead,
    Alive,
}

impl Default for Cell {
    fn default() -> Self {
        Self::Dead
    }
}

#[derive(PartialEq, Eq)]
pub struct GameOfLife {
    grid: Grid<Cell>,
}

impl GameOfLife {
    pub fn from_grid(grid: Grid<Cell>) -> Self {
        Self { grid }
    }

    pub fn get_grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    pub fn step(&mut self) {
        let (rows, cols) = self.grid.size();
        let tmp_grid = Grid::<Cell>::from_slice(self.grid.get_grid(), rows, cols);

        for row in 0..rows {
            for col in 0..cols {
                self.grid
                    .set(GameOfLife::new_status(&tmp_grid, row, col), row, col);
            }
        }
    }

    pub fn new_status(grid: &Grid<Cell>, row: usize, col: usize) -> Cell {
        let alive_count = Self::get_alive_count(grid, &grid.neighbours(row, col));
        match (grid.get(row, col), alive_count) {
            (&Cell::Alive, 2) => Cell::Alive,
            (&Cell::Alive, 3) => Cell::Alive,
            (&Cell::Dead, 3) => Cell::Alive,
            (_, _) => Cell::Dead,
        }
    }

    pub fn get_alive_count(grid: &Grid<Cell>, neighbours: &[(usize, usize)]) -> usize {
        let mut count: usize = 0;
        for coords in neighbours {
            if grid.get(coords.0, coords.1) == &Cell::Alive {
                count += 1;
            }
        }
        count
    }
}
