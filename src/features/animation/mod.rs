use crate::app::state::GridState;
use crate::features::image::loader::LoadedImage;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    pub const AUTHORED: [Direction; 3] = [Direction::Up, Direction::Down, Direction::Right];
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Right,
        Direction::Left,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Direction::Up => "Up",
            Direction::Down => "Down",
            Direction::Right => "Right",
            Direction::Left => "Left",
        }
    }

    pub fn is_authored(self) -> bool {
        self != Direction::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Playback {
    Loop,
    LoopN { times: u16 },
    OneShot { hold_last: bool },
}

impl Default for Playback {
    fn default() -> Self {
        Self::Loop
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeriveRule {
    MirrorX { from: Direction, to: Direction },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub cell: u16,
    pub ms: u16,
    pub flip_x: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub display: String,
    pub category: String,
    pub playback: Playback,
    pub tracks: BTreeMap<Direction, Track>,
    pub derives: Vec<DeriveRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridMeta {
    pub cell_w: u32,
    pub cell_h: u32,
    pub offset_x: u32,
    pub offset_y: u32,
    pub spacing_x: u32,
    pub spacing_y: u32,
    pub columns: u32,
    pub rows: u32,
}

impl GridMeta {
    pub fn from_grid(grid: &GridState) -> Self {
        Self {
            cell_w: grid.cell_width,
            cell_h: grid.cell_height,
            offset_x: grid.offset_x,
            offset_y: grid.offset_y,
            spacing_x: 0,
            spacing_y: 0,
            columns: grid.columns,
            rows: grid.rows,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetMeta {
    pub image: String,
    pub grid: GridMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultsMeta {
    pub frame_ms: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartMapping {
    FollowBodyCells,
    CustomOverride {
        clips: BTreeMap<String, BTreeMap<Direction, Track>>,
    },
}

impl Default for PartMapping {
    fn default() -> Self {
        Self::FollowBodyCells
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LayersMeta {
    #[serde(default)]
    pub equipped_parts: Vec<String>,
    #[serde(default)]
    pub mappings: BTreeMap<String, PartMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimProject {
    pub version: u32,
    pub sheet: SheetMeta,
    pub defaults: DefaultsMeta,
    #[serde(default)]
    pub layers: LayersMeta,
    pub clips: Vec<Clip>,
}

impl AnimProject {
    pub fn seeded(image_path: String, grid: &GridState) -> Self {
        let defaults = DefaultsMeta { frame_ms: 120 };
        Self {
            version: 1,
            sheet: SheetMeta {
                image: image_path,
                grid: GridMeta::from_grid(grid),
            },
            defaults,
            layers: LayersMeta::default(),
            clips: default_seed_catalog()
                .into_iter()
                .map(|(category, id, display)| Clip {
                    id: id.to_string(),
                    display: display.to_string(),
                    category: category.to_string(),
                    playback: Playback::Loop,
                    tracks: default_authored_tracks(),
                    derives: vec![DeriveRule::MirrorX {
                        from: Direction::Right,
                        to: Direction::Left,
                    }],
                })
                .collect(),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct AnimationAuthoringState {
    pub project: AnimProject,
    pub active_clip: usize,
    pub active_direction: Direction,
    pub active_step: Option<usize>,
    pub save_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellToggleResult {
    Added,
    RemovedLast,
}

impl Default for AnimationAuthoringState {
    fn default() -> Self {
        let grid = GridState::default();
        Self {
            project: AnimProject::seeded("assets/body.png".to_string(), &grid),
            active_clip: 0,
            active_direction: Direction::Right,
            active_step: None,
            save_path: None,
        }
    }
}

impl AnimationAuthoringState {
    pub fn sync_sheet_meta(&mut self, grid: &GridState, loaded: &LoadedImage) {
        self.project.sheet.grid = GridMeta::from_grid(grid);
        if let Some(image) = loaded.asset_path.as_deref() {
            self.project.sheet.image = image.to_string();
        }
    }

    pub fn active_clip(&self) -> Option<&Clip> {
        self.project.clips.get(self.active_clip)
    }

    pub fn active_clip_mut(&mut self) -> Option<&mut Clip> {
        self.project.clips.get_mut(self.active_clip)
    }

    pub fn active_track(&self) -> Option<&Track> {
        let clip = self.active_clip()?;
        if self.active_direction == Direction::Left {
            return derived_left_track(clip);
        }
        clip.tracks.get(&self.active_direction)
    }

    pub fn active_track_mut(&mut self) -> Option<&mut Track> {
        if !self.active_direction.is_authored() {
            return None;
        }
        let direction = self.active_direction;
        let clip = self.active_clip_mut()?;
        Some(clip.tracks.entry(direction).or_default())
    }

    pub fn set_active_clip(&mut self, clip_index: usize) {
        self.active_clip = clip_index.min(self.project.clips.len().saturating_sub(1));
        self.active_step = None;
    }

    pub fn toggle_cell_in_active_track(&mut self, cell: usize) -> Option<CellToggleResult> {
        let cell_u16 = u16::try_from(cell).ok()?;
        let default_ms = self.project.defaults.frame_ms;
        let track = self.active_track_mut()?;

        if let Some(last_index) = track.steps.iter().rposition(|step| step.cell == cell_u16) {
            track.steps.remove(last_index);
            Some(CellToggleResult::RemovedLast)
        } else {
            track.steps.push(Step {
                cell: cell_u16,
                ms: default_ms,
                flip_x: false,
            });
            Some(CellToggleResult::Added)
        }
    }

    pub fn active_track_unique_cells(&self) -> BTreeSet<usize> {
        let mut cells = BTreeSet::new();
        if let Some(track) = self.active_track() {
            for step in &track.steps {
                cells.insert(step.cell as usize);
            }
        }
        cells
    }

    pub fn append_mirrored_copy(&mut self) -> usize {
        let Some(track) = self.active_track_mut() else {
            return 0;
        };
        let original = track.steps.clone();
        for step in &original {
            track.steps.push(Step {
                cell: step.cell,
                ms: step.ms,
                flip_x: true,
            });
        }
        original.len()
    }

    pub fn to_ron_string(&self) -> Result<String, String> {
        ron::ser::to_string_pretty(&self.project, ron::ser::PrettyConfig::default())
            .map_err(|err| format!("Failed to serialize animation project: {err}"))
    }

    pub fn load_from_ron_str(&mut self, ron_text: &str) -> Result<(), String> {
        let parsed: AnimProject = ron::from_str(ron_text)
            .map_err(|err| format!("Failed to parse animation project RON: {err}"))?;
        if parsed.clips.is_empty() {
            return Err("Animation project must contain at least one clip".to_string());
        }
        self.project = parsed;
        self.active_clip = 0;
        self.active_direction = Direction::Right;
        self.active_step = None;
        Ok(())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let ron_text = self.to_ron_string()?;
        std::fs::write(path, ron_text).map_err(|err| {
            format!(
                "Failed to write animation project to {}: {err}",
                path.display()
            )
        })
    }

    pub fn load_from_path(&mut self, path: &Path) -> Result<(), String> {
        let contents = std::fs::read_to_string(path).map_err(|err| {
            format!(
                "Failed to read animation project from {}: {err}",
                path.display()
            )
        })?;
        self.load_from_ron_str(&contents)?;
        self.save_path = Some(path.to_path_buf());
        Ok(())
    }
}

fn default_authored_tracks() -> BTreeMap<Direction, Track> {
    let mut tracks = BTreeMap::new();
    for direction in Direction::AUTHORED {
        tracks.insert(direction, Track::default());
    }
    tracks
}

fn derived_left_track(clip: &Clip) -> Option<&Track> {
    for rule in &clip.derives {
        match rule {
            DeriveRule::MirrorX {
                from: Direction::Right,
                to: Direction::Left,
            } => {
                let right = clip.tracks.get(&Direction::Right)?;
                return Some(right);
            }
            _ => {}
        }
    }
    None
}

pub struct AnimationFeaturePlugin;

impl Plugin for AnimationFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationAuthoringState>();
    }
}

fn default_seed_catalog() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("Locomotion", "walk", "Walk"),
        ("Locomotion", "run", "Run"),
        ("Locomotion", "jump", "Jump"),
        ("Locomotion", "climb", "Climb"),
        ("Locomotion", "mount_up", "Mount Up"),
        ("Locomotion", "ride_mount", "Ride Mount"),
        ("Locomotion", "ride_mount_drunk", "Ride Mount Drunk"),
        ("Carry", "pickup", "Pickup"),
        ("Carry", "carry", "Carry"),
        ("Carry", "walk_carry", "Walk Carry"),
        ("Carry", "run_carry", "Run Carry"),
        ("Carry", "jump_carry", "Jump Carry"),
        ("Carry", "putdown", "Putdown"),
        ("Interaction", "push", "Push"),
        ("Interaction", "pull", "Pull"),
        ("Interaction", "plant_seed", "Plant Seed"),
        ("Interaction", "water", "Water"),
        ("Interaction", "work_station", "Work Station"),
        ("Performance / Social", "sing", "Sing"),
        ("Performance / Social", "play_guitar", "Play Guitar"),
        ("Performance / Social", "play_flute", "Play Flute"),
        ("Performance / Social", "play_drums", "Play Drums"),
        ("Performance / Social", "wave", "Wave"),
        ("Performance / Social", "hug", "Hug"),
        ("Performance / Social", "thumbs_up", "Thumbs Up"),
        ("Performance / Social", "sniff", "Sniff"),
        ("Performance / Social", "sad", "Sad"),
        ("Performance / Social", "shocked", "Shocked"),
        ("Performance / Social", "laugh", "Laugh"),
        ("Performance / Social", "impatient", "Impatient"),
        ("Performance / Social", "mad_stomp", "Mad Stomp"),
        ("Fishing", "fish_cast", "Fish: Cast"),
        ("Fishing", "fish_wait_bite", "Fish: Wait Bite"),
        ("Fishing", "fish_catch", "Fish: Catch"),
        ("Combat", "strike_overhand", "Strike Overhand"),
        ("Combat", "strike_forehand", "Strike Forehand"),
        ("Combat", "strike_backhand", "Strike Backhand"),
        ("Combat", "bow_shot", "Bow Shot"),
        ("Combat", "hurt", "Hurt"),
        ("Combat", "evade", "Evade"),
        ("Animals", "pet_dog", "Pet Dog"),
        ("Animals", "milk_cow", "Milk Cow"),
        ("Animals", "pet_horse", "Pet Horse"),
        ("Poses", "idle", "Idle"),
        ("Poses", "sit_floor", "Sit Floor"),
        ("Poses", "sit_ledge", "Sit Ledge"),
        ("Poses", "sit_chair", "Sit Chair"),
        ("Poses", "meditate", "Meditate"),
        ("Poses", "sleep", "Sleep"),
        ("Poses", "sleep_sit_chair", "Sleep Sit Chair"),
        ("Drinking", "drink_stand", "Drink Stand"),
        ("Drinking", "drink_sit", "Drink Sit"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_project_contains_walk_clip() {
        let project = AnimProject::seeded("assets/body.png".to_string(), &GridState::default());
        let walk = project.clips.iter().find(|clip| clip.id == "walk");
        assert!(walk.is_some());
    }

    #[test]
    fn ron_round_trip_preserves_steps() {
        let mut state = AnimationAuthoringState::default();
        state.project.clips[0]
            .tracks
            .entry(Direction::Right)
            .or_default()
            .steps
            .push(Step {
                cell: 12,
                ms: 140,
                flip_x: false,
            });

        let encoded = state.to_ron_string().expect("serialize");
        let mut decoded = AnimationAuthoringState::default();
        decoded.load_from_ron_str(&encoded).expect("deserialize");

        let step = decoded.project.clips[0].tracks[&Direction::Right].steps[0];
        assert_eq!(step.cell, 12);
        assert_eq!(step.ms, 140);
        assert!(!step.flip_x);
    }

    #[test]
    fn ron_round_trip_preserves_grid_defaults_and_playback_modes() {
        let mut state = AnimationAuthoringState::default();
        state.project.sheet.image = "assets/sheets/custom_body.png".to_string();
        state.project.sheet.grid = GridMeta {
            cell_w: 64,
            cell_h: 64,
            offset_x: 2,
            offset_y: 4,
            spacing_x: 0,
            spacing_y: 0,
            columns: 16,
            rows: 12,
        };
        state.project.defaults.frame_ms = 95;
        state.project.layers.equipped_parts = vec![
            "01body/human/00".to_string(),
            "05shrt/tanktop/00a".to_string(),
            "14head/headscarf/00b/e".to_string(),
        ];
        state.project.layers.mappings.insert(
            "05shrt/tanktop/00a".to_string(),
            PartMapping::FollowBodyCells,
        );

        if let Some(clip) = state.project.clips.get_mut(0) {
            clip.playback = Playback::LoopN { times: 3 };
            clip.tracks
                .entry(Direction::Up)
                .or_default()
                .steps
                .push(Step {
                    cell: 33,
                    ms: 111,
                    flip_x: true,
                });
        }
        if let Some(clip) = state.project.clips.get_mut(1) {
            clip.playback = Playback::OneShot { hold_last: true };
            clip.tracks
                .entry(Direction::Down)
                .or_default()
                .steps
                .push(Step {
                    cell: 9,
                    ms: 87,
                    flip_x: false,
                });
        }

        let encoded = state.to_ron_string().expect("serialize");
        let mut decoded = AnimationAuthoringState::default();
        decoded.load_from_ron_str(&encoded).expect("deserialize");

        assert_eq!(decoded.project.sheet.image, "assets/sheets/custom_body.png");
        assert_eq!(decoded.project.sheet.grid.columns, 16);
        assert_eq!(decoded.project.sheet.grid.rows, 12);
        assert_eq!(decoded.project.defaults.frame_ms, 95);
        assert_eq!(decoded.project.layers.equipped_parts.len(), 3);
        assert!(
            decoded
                .project
                .layers
                .equipped_parts
                .contains(&"14head/headscarf/00b/e".to_string())
        );
        assert!(matches!(
            decoded.project.layers.mappings.get("05shrt/tanktop/00a"),
            Some(PartMapping::FollowBodyCells)
        ));
        assert!(matches!(
            decoded.project.clips[0].playback,
            Playback::LoopN { times: 3 }
        ));
        assert!(matches!(
            decoded.project.clips[1].playback,
            Playback::OneShot { hold_last: true }
        ));
        let clip0_step = decoded.project.clips[0].tracks[&Direction::Up].steps[0];
        assert_eq!(clip0_step.cell, 33);
        assert_eq!(clip0_step.ms, 111);
        assert!(clip0_step.flip_x);
    }

    #[test]
    fn load_legacy_ron_without_layers_meta() {
        let legacy = r#"
(
    version: 1,
    sheet: (
        image: "assets/body.png",
        grid: (
            cell_w: 32,
            cell_h: 32,
            offset_x: 0,
            offset_y: 0,
            spacing_x: 0,
            spacing_y: 0,
            columns: 8,
            rows: 8,
        ),
    ),
    defaults: (frame_ms: 120),
    clips: [
        (
            id: "idle",
            display: "Idle",
            category: "Poses",
            playback: Loop,
            tracks: {
                Up: (steps: []),
                Down: (steps: []),
                Right: (steps: []),
            },
            derives: [MirrorX(from: Right, to: Left)],
        ),
    ],
)
"#;

        let mut state = AnimationAuthoringState::default();
        state.load_from_ron_str(legacy).expect("legacy parse");
        assert!(state.project.layers.equipped_parts.is_empty());
        assert!(state.project.layers.mappings.is_empty());
    }
}
