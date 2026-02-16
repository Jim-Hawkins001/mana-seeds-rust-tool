use bevy::prelude::*;

#[derive(Resource, Debug, Default, Clone)]
pub struct LoadedImage {
    pub asset_path: Option<String>,
    pub name: Option<String>,
    pub handle: Option<Handle<Image>>,
    pub size: Option<UVec2>,
    pub status: Option<String>,
}

impl LoadedImage {
    pub fn set_pending(&mut self, asset_path: String, name: String, handle: Handle<Image>) {
        self.asset_path = Some(asset_path);
        self.name = Some(name);
        self.handle = Some(handle);
        self.size = None;
        self.status = Some("Loading image...".to_string());
    }
}

pub struct ImageFeaturePlugin;

impl Plugin for ImageFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedImage>()
            .add_systems(Update, sync_loaded_image_size);
    }
}

fn sync_loaded_image_size(images: Res<Assets<Image>>, mut loaded: ResMut<LoadedImage>) {
    let Some(handle) = loaded.handle.as_ref() else {
        return;
    };
    let Some(image) = images.get(handle) else {
        return;
    };
    let size = image.size();
    if loaded.size == Some(size) {
        return;
    }
    loaded.size = Some(size);
    loaded.status = Some(format!("Loaded {}x{}", size.x, size.y));
}
