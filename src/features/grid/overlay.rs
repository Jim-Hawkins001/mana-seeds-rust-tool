use crate::app::state::GridState;
use bevy::prelude::*;

#[derive(Component)]
pub struct GridOverlayMarker;

pub fn grid_line_color() -> Color {
    Color::srgba(0.75, 0.75, 0.75, 0.45)
}

pub fn selected_fill_color() -> Color {
    Color::srgba(0.2, 0.8, 0.95, 0.35)
}

pub fn selected_border_color() -> Color {
    Color::srgba(0.2, 0.9, 1.0, 0.9)
}

pub fn grid_dimensions(grid: &GridState) -> Vec2 {
    Vec2::new(
        grid.columns as f32 * grid.cell_width as f32,
        grid.rows as f32 * grid.cell_height as f32,
    )
}
