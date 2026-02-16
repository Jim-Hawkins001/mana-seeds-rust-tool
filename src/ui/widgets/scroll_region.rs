use bevy::prelude::*;

#[derive(Component)]
pub struct ScrollRegion;

pub fn scroll_node() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        ..default()
    }
}
