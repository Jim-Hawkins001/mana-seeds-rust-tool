use bevy::prelude::*;

#[derive(Component)]
pub struct VerticalSplitter;

#[derive(Component)]
pub struct HorizontalSplitter;

pub const SPLITTER_SIZE: f32 = 4.0;

pub fn splitter_color() -> BackgroundColor {
    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08))
}
