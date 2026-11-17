use crate::unit::Unit;

/// A rectangular grid where each cell holds at most one unit.
#[derive(Clone)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Option<Unit>>,
}

impl Board {
    /// An empty board of `width` columns by `height` rows.
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    /// Number of columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Position of `(row, col)` in the cell vector, or `None` if off the board.
    pub(crate) fn index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.height && col < self.width {
            return Some(row * self.width + col);
        }
        None
    }

    /// The unit at `(row, col)`. `None` if the cell is empty or off the board.
    pub fn get(&self, row: usize, col: usize) -> Option<Unit> {
        let i = self.index(row, col)?;
        self.cells[i]
    }

    /// Puts `unit` (or clears the cell with `None`). Returns `false` if off the board.
    pub fn set(&mut self, row: usize, col: usize, unit: Option<Unit>) -> bool {
        let Some(i) = self.index(row, col) else {
            return false;
        };
        self.cells[i] = unit;
        true
    }

    /// In-bounds cells touching `(row, col)` (up to 8), in row-major order.
    /// The cell itself is excluded.
    pub fn neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut cells = Vec::with_capacity(8);
        for r in row.saturating_sub(1)..=row + 1 {
            for c in col.saturating_sub(1)..=col + 1 {
                if (r, c) != (row, col) && self.index(r, c).is_some() {
                    cells.push((r, c));
                }
            }
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::{Team, Unit, UnitKind};

    fn red() -> Unit {
        Unit {
            team: Team::Red,
            kind: UnitKind::Basic,
            health: UnitKind::Basic.stats().max_health,
        }
    }

    #[test]
    fn new_board_is_empty() {
        let board = Board::new(3, 3);
        assert_eq!(board.get(1, 1), None);
    }

    #[test]
    fn set_then_get_returns_unit() {
        let mut board = Board::new(3, 3);
        assert!(board.set(0, 0, Some(red())));
        assert_eq!(board.get(0, 0), Some(red()));
    }

    #[test]
    fn set_touches_only_its_own_cell() {
        let mut board = Board::new(3, 4);
        let target_row = 1;
        let target_col = 2;
        assert!(board.set(target_row, target_col, Some(red())));
        for row in 0..board.height {
            for col in 0..board.width {
                if row == target_row && col == target_col {
                    assert_eq!(board.get(row, col), Some(red()));
                } else {
                    assert_eq!(board.get(row, col), None);
                }
            }
        }
    }

    #[test]
    fn set_none_clears_cell() {
        let mut board = Board::new(3, 3);
        assert!(board.set(1, 2, Some(red())));
        assert_eq!(board.get(1, 2), Some(red()));
        assert!(board.set(1, 2, None));
        assert_eq!(board.get(1, 2), None);
    }

    #[test]
    fn last_cell_is_valid() {
        let mut board = Board::new(3, 3);
        assert!(board.set(2, 2, Some(red())));
        assert_eq!(board.get(2, 2), Some(red()));
    }

    #[test]
    fn out_of_bounds_get_is_none() {
        let board = Board::new(3, 3);
        assert_eq!(board.get(3, 0), None);
        assert_eq!(board.get(0, 3), None);
    }

    #[test]
    fn neighbors_corner_has_three() {
        let board = Board::new(4, 4);
        assert_eq!(board.neighbors(0, 0), vec![(0, 1), (1, 0), (1, 1)]);
        assert_eq!(board.neighbors(3, 3).len(), 3);
    }

    #[test]
    fn neighbors_edge_has_five() {
        let board = Board::new(4, 4);
        assert_eq!(board.neighbors(0, 2).len(), 5);
        assert_eq!(board.neighbors(2, 0).len(), 5);
        assert_eq!(board.neighbors(3, 1).len(), 5);
        assert_eq!(board.neighbors(1, 3).len(), 5);
    }

    #[test]
    fn neighbors_middle_has_eight() {
        let board = Board::new(4, 4);
        assert_eq!(
            board.neighbors(1, 1),
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 2),
                (2, 0),
                (2, 1),
                (2, 2)
            ]
        );
    }

    #[test]
    fn neighbors_excludes_self() {
        let board = Board::new(3, 3);
        assert!(!board.neighbors(1, 1).contains(&(1, 1)));
    }

    #[test]
    fn out_of_bounds_set_fails() {
        let mut board = Board::new(3, 4);
        assert!(!board.set(4, 0, Some(red())));
        assert!(!board.set(0, 3, Some(red())));
        for row in 0..board.height {
            for col in 0..board.width {
                assert_eq!(board.get(row, col), None);
            }
        }
    }
}
