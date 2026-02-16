#[cfg(test)]
use crate::app::state::{GridState, SelectionState};

#[cfg(test)]
use super::state;

#[cfg(test)]
pub fn toggle_cell(selection: &mut SelectionState, grid: &GridState, index: usize) {
    if !state::is_valid_cell(grid, index) {
        return;
    }

    if !selection.selected_cells.insert(index) {
        selection.selected_cells.remove(&index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_adds_and_removes() {
        let grid = GridState {
            rows: 1,
            columns: 2,
            ..Default::default()
        };
        let mut selection = SelectionState::default();

        toggle_cell(&mut selection, &grid, 1);
        assert!(selection.selected_cells.contains(&1));

        toggle_cell(&mut selection, &grid, 1);
        assert!(!selection.selected_cells.contains(&1));
    }
}
