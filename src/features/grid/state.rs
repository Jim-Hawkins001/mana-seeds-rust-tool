use crate::app::state::GridState;

pub fn cell_index(row: u32, column: u32, columns: u32) -> usize {
    (row * columns + column) as usize
}

pub fn cell_count(grid: &GridState) -> usize {
    (grid.rows * grid.columns) as usize
}

#[cfg(test)]
pub fn is_valid_cell(grid: &GridState, index: usize) -> bool {
    index < cell_count(grid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::GridState;

    #[test]
    fn index_is_row_major() {
        assert_eq!(cell_index(0, 0, 4), 0);
        assert_eq!(cell_index(0, 3, 4), 3);
        assert_eq!(cell_index(1, 0, 4), 4);
        assert_eq!(cell_index(2, 1, 4), 9);
    }

    #[test]
    fn valid_cell_uses_grid_dimensions() {
        let grid = GridState {
            rows: 2,
            columns: 3,
            ..Default::default()
        };
        assert!(is_valid_cell(&grid, 5));
        assert!(!is_valid_cell(&grid, 6));
    }
}
