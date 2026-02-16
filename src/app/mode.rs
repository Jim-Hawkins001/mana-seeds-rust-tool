use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EditorMode {
    #[default]
    Animations,
    Parts,
    Outfits,
}

impl EditorMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Animations => "Animations",
            Self::Parts => "Parts",
            Self::Outfits => "Outfits",
        }
    }
}
