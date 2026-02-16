use crate::app::mode::EditorMode;
use bevy::prelude::*;

pub fn toolbar_node(height: f32) -> Node {
    Node {
        width: percent(100),
        height: px(height),
        padding: UiRect::axes(px(12), px(8)),
        align_items: AlignItems::Center,
        column_gap: px(12),
        overflow: Overflow::visible(),
        ..default()
    }
}

pub fn toolbar_bg() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.09, 0.09, 0.11))
}

pub fn mode_text(mode: EditorMode) -> String {
    format!("Mode: {}", mode.label())
}
