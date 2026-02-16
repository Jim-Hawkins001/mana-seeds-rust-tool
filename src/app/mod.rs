pub mod mode;
pub mod state;

use crate::features::EditorFeaturesPlugin;
use crate::ui::EditorUiPlugin;
use bevy::prelude::*;
use mode::EditorMode;
use state::{GridState, LayoutState, SelectionState};

pub struct EditorAppPlugin;

impl Plugin for EditorAppPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorMode>()
            .init_resource::<GridState>()
            .init_resource::<SelectionState>()
            .init_resource::<LayoutState>()
            .add_plugins((EditorFeaturesPlugin, EditorUiPlugin));
    }
}
