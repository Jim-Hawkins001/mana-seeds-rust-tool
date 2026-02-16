pub mod panels;
pub mod shell;
pub mod toolbar;
pub mod widgets;

use bevy::prelude::*;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(shell::UiShellPlugin);
    }
}
