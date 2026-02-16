use bevy::prelude::*;

pub fn node(width: f32) -> Node {
    Node {
        width: px(width),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(12)),
        ..default()
    }
}

pub fn background() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.11, 0.11, 0.13))
}
