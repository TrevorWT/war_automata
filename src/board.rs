use crate::unit::Unit;

/// todo
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Option<Unit>>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.height && col < self.width {
            return Some(row * self.width + col);
        }
        None
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Unit> {
        let i = self.index(row, col)?;
        self.cells[i]
    }

    pub fn set(&mut self, row: usize, col: usize, unit: Option<Unit>) -> bool {
        let Some(i) = self.index(row, col) else {
            return false;
        };
        self.cells[i] = unit;
        true
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
