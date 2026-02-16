use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Debug, Clone)]
pub struct GridState {
    pub rows: u32,
    pub columns: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub offset_x: u32,
    pub offset_y: u32,
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            rows: 8,
            columns: 8,
            cell_width: 32,
            cell_height: 32,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

impl GridState {
    pub fn normalize(&mut self) {
        self.rows = self.rows.max(1);
        self.columns = self.columns.max(1);
        self.cell_width = self.cell_width.max(1);
        self.cell_height = self.cell_height.max(1);
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct SelectionState {
    pub selected_cells: HashSet<usize>,
}

#[derive(Resource, Debug, Clone)]
pub struct LayoutState {
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,
    pub toolbar_height: f32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            left_panel_width: 280.0,
            right_panel_width: 280.0,
            bottom_panel_height: 180.0,
            toolbar_height: 44.0,
        }
    }
}

impl LayoutState {
    pub const MIN_SIDE_PANEL_WIDTH: f32 = 180.0;
    pub const MIN_BOTTOM_PANEL_HEIGHT: f32 = 120.0;
}
