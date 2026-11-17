/// todo
pub fn distance(a: (usize, usize), b: (usize, usize)) -> usize {
    let row_gap = a.0.abs_diff(b.0);
    let col_gap = a.1.abs_diff(b.1);
    row_gap.max(col_gap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_line() {
        assert_eq!(distance((0, 0), (3, 0)), 3);
    }

    #[test]
    fn diagonal() {
        assert_eq!(distance((2, 2), (4, 4)), 2);
    }

    #[test]
    fn max_gap() {
        assert_eq!(distance((2, 2), (4, 5)), 3);
    }

    #[test]
    fn same_cell() {
        assert_eq!(distance((2, 2), (2, 2)), 0);
    }
}
