use crate::features::layers::LayerCode;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OutfitDb {
    pub version: u32,
    pub outfits: Vec<Outfit>,
}

impl Default for OutfitDb {
    fn default() -> Self {
        Self {
            version: 1,
            outfits: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Outfit {
    pub outfit_id: String,
    pub display_name: String,
    pub tags: Vec<String>,
    pub equipped: Vec<EquippedPart>,
    pub palette: PaletteSelection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EquippedPart {
    pub layer: LayerCode,
    pub part_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaletteSelection {
    pub skin: RampChoice,
    pub hair: RampChoice,
    pub outfit_main: RampChoice,
    pub outfit_accent: Option<RampChoice>,
}

impl Default for PaletteSelection {
    fn default() -> Self {
        Self {
            skin: RampChoice::Preset("palette:0:variant:0".to_string()),
            hair: RampChoice::Preset("palette:0:variant:0".to_string()),
            outfit_main: RampChoice::Preset("palette:0:variant:0".to_string()),
            outfit_accent: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RampChoice {
    Preset(String),
    Custom(Vec<[u8; 4]>),
}

#[derive(Resource, Debug, Clone)]
pub struct OutfitDbState {
    pub db: OutfitDb,
    pub path: PathBuf,
    pub selected: Option<usize>,
    pub dirty: bool,
    pub loaded: bool,
    pub last_error: Option<String>,
}

impl Default for OutfitDbState {
    fn default() -> Self {
        Self {
            db: OutfitDb::default(),
            path: default_outfits_path(),
            selected: None,
            dirty: false,
            loaded: false,
            last_error: None,
        }
    }
}

impl OutfitDbState {
    pub fn load(&mut self) -> Result<(), String> {
        self.load_from_path(self.path.clone())
    }

    pub fn load_from_path(&mut self, path: PathBuf) -> Result<(), String> {
        self.path = path.clone();
        let contents = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.db = OutfitDb::default();
                self.selected = None;
                self.dirty = false;
                self.loaded = true;
                self.last_error = None;
                return Ok(());
            }
            Err(err) => {
                let message = format!("Failed to read {}: {err}", path.display());
                self.last_error = Some(message.clone());
                self.loaded = true;
                return Err(message);
            }
        };

        let parsed: OutfitDb = ron::from_str(&contents)
            .map_err(|err| format!("Failed to parse {}: {err}", path.display()))?;
        self.db = parsed;
        self.selected = self
            .selected
            .filter(|index| *index < self.db.outfits.len())
            .or_else(|| (!self.db.outfits.is_empty()).then_some(0));
        self.dirty = false;
        self.loaded = true;
        self.last_error = None;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), String> {
        self.save_to_path(self.path.clone())
    }

    pub fn save_to_path(&mut self, path: PathBuf) -> Result<(), String> {
        self.path = path.clone();
        let ron_text = ron::ser::to_string_pretty(&self.db, ron::ser::PrettyConfig::default())
            .map_err(|err| format!("Failed to serialize outfits db: {err}"))?;
        std::fs::write(&path, ron_text)
            .map_err(|err| format!("Failed to write {}: {err}", path.display()))?;
        self.dirty = false;
        self.last_error = None;
        Ok(())
    }
}

fn default_outfits_path() -> PathBuf {
    project_root_path("outfits.ron")
}

fn project_root_path(file_name: &str) -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let direct = cwd.join(file_name);
        if direct.parent().is_some_and(Path::exists) {
            return direct;
        }
        let nested = cwd.join("asset_hander").join(file_name);
        if nested.parent().is_some_and(Path::exists) {
            return nested;
        }
    }
    PathBuf::from(file_name)
}

pub struct OutfitsFeaturePlugin;

impl Plugin for OutfitsFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutfitDbState>()
            .add_systems(Startup, load_outfits_db_once);
    }
}

fn load_outfits_db_once(mut outfits: ResMut<OutfitDbState>) {
    let _ = outfits.load();
}
