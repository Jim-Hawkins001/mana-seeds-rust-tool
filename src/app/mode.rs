use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EditorMode {
    #[default]
    Sprite,
}

impl EditorMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sprite => "Sprite Mode",
        }
    }
}
