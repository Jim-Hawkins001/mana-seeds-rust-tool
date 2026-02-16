pub mod animation;
pub mod grid;
pub mod image;
pub mod layers;
pub mod outfits;

use bevy::prelude::*;

pub struct EditorFeaturesPlugin;

impl Plugin for EditorFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            image::ImageFeaturePlugin,
            animation::AnimationFeaturePlugin,
            layers::LayersFeaturePlugin,
            outfits::OutfitsFeaturePlugin,
        ));
    }
}
