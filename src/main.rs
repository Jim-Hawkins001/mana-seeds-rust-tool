mod app;
mod features;
mod ui;

use app::EditorAppPlugin;
use bevy::input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin};
use bevy::prelude::*;
use bevy_ui_widgets::ScrollbarPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Mana Seeds Rust Tool".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            ScrollbarPlugin,
            InputDispatchPlugin,
            TabNavigationPlugin,
        ))
        .add_plugins(EditorAppPlugin)
        .run();
}
