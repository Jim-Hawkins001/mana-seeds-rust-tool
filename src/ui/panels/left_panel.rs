use bevy::prelude::*;

pub const TITLE: &str = "Sprite Asset Panel";

pub fn node(width: f32) -> Node {
    Node {
        width: px(width),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(12)),
        row_gap: px(8),
        ..default()
    }
}

pub fn background() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.11, 0.11, 0.13))
}
