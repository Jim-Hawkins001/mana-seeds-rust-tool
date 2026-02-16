use bevy::prelude::*;

pub const TITLE: &str = "Layers";

pub fn node(height: f32) -> Node {
    Node {
        width: percent(100),
        height: px(height),
        padding: UiRect::all(px(12)),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}
