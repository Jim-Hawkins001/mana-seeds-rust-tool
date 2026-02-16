use crate::app::mode::EditorMode;
use crate::app::state::{GridState, LayoutState, SelectionState};
use crate::features::animation::{
    AnimationAuthoringState, CellToggleResult, Clip, DeriveRule, Direction, PartMapping, Playback,
    Track,
};
use crate::features::grid::{overlay, state as grid_state};
use crate::features::image::loader::LoadedImage;
use crate::features::layers::{LayerCode, PaperDollState};
use crate::ui::panels::{bottom_panel, center_canvas, left_panel, right_panel};
use crate::ui::toolbar as top_toolbar;
use crate::ui::widgets::{scroll_region, splitters};
use bevy::app::AppExit;
use bevy::image::{TextureAtlas, TextureAtlasLayout};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, UiGlobalTransform};
use bevy_ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};
use rfd::FileDialog;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

mod animation_edit;
mod interaction;
mod setup;
mod sync;
mod toolbar;

use self::animation_edit::*;
use self::interaction::*;
use self::setup::*;
use self::sync::*;
use self::toolbar::*;

pub struct UiShellPlugin;

impl Plugin for UiShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasUiState>()
            .init_resource::<CanvasView>()
            .init_resource::<InputContext>()
            .init_resource::<LeftPanelSectionsState>()
            .init_resource::<PalettePanelState>()
            .init_resource::<LayerPaletteState>()
            .init_resource::<PaletteRemapCache>()
            .init_resource::<GridFieldDrafts>()
            .init_resource::<AnimationFieldDrafts>()
            .init_resource::<AnimationPanelState>()
            .init_resource::<PreviewStripUiState>()
            .init_resource::<PreviewAtlasState>()
            .init_resource::<AnimViewerState>()
            .init_resource::<CanvasPanState>()
            .init_resource::<DragState>()
            .init_resource::<ToolbarUiState>()
            .add_systems(Startup, setup_ui_shell)
            .add_systems(
                Update,
                (
                    handle_toolbar_menu_toggles,
                    handle_toolbar_menu_actions,
                    sync_toolbar_menu_visibility,
                    handle_open_image_button,
                    handle_load_project_button,
                    handle_save_project_button,
                    handle_clear_selection_button,
                    handle_left_panel_section_toggles,
                    sync_left_panel_sections,
                    handle_grid_field_focus,
                    handle_grid_field_keyboard_input,
                    commit_grid_field_on_blur,
                    sync_grid_fields_from_state,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_animation_navigation_buttons,
                    handle_animation_data_buttons,
                    handle_animation_tree_clicks,
                    handle_part_cycle_buttons,
                    handle_part_palette_cycle_buttons,
                    handle_palette_cycle_buttons,
                    handle_preview_step_buttons,
                    handle_animation_field_focus,
                    handle_animation_field_keyboard_input,
                    commit_animation_field_on_blur,
                    handle_anim_viewer_controls,
                    update_hovered_region,
                    update_input_capture,
                    route_wheel_input,
                    handle_canvas_interaction,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_splitter_press,
                    apply_splitter_drag,
                    update_panel_sizes,
                    mark_canvas_dirty_on_data_change,
                    rebuild_canvas_if_needed,
                    sync_left_panel_texts,
                    sync_parts_panel_text,
                    sync_grid_field_widgets,
                    sync_animation_field_widgets,
                    sync_animation_tree_visibility,
                    sync_direction_button_styles,
                    sync_mode_text,
                    sync_selection_text,
                    sync_animation_text,
                    sync_preview_atlas_layout,
                    mark_preview_strip_dirty,
                    rebuild_preview_strip_if_needed,
                    sync_cell_visuals,
                    sync_animation_project_sheet,
                    sync_workspace_highlights_derived,
                ),
            )
            .add_systems(Update, sync_palette_panel_text)
            .add_systems(
                Update,
                (
                    sync_anim_viewer_clip_binding,
                    tick_anim_viewer_playback,
                    sync_anim_viewer_display,
                    sync_anim_viewer_button_styles,
                ),
            );
    }
}

#[derive(Resource, Default)]
struct CanvasUiState {
    dirty: bool,
    dynamic_entities: Vec<Entity>,
}

#[derive(Resource, Debug, Clone)]
struct CanvasView {
    offset: Vec2,
    zoom: f32,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

#[derive(Resource, Default)]
struct CanvasPanState {
    dragging: bool,
    last_window_cursor: Option<Vec2>,
}

#[derive(Resource, Default)]
struct DragState {
    active: Option<SplitterKind>,
    last_cursor: Option<Vec2>,
}

#[derive(Component)]
struct FileMenuToggleButton;
#[derive(Component)]
struct ModeMenuToggleButton;
#[derive(Component)]
struct FileMenuPanel;
#[derive(Component)]
struct ModeMenuPanel;
#[derive(Component)]
struct FileOpenMenuItem;
#[derive(Component)]
struct FileLoadProjectMenuItem;
#[derive(Component)]
struct FileSaveMenuItem;
#[derive(Component)]
struct FileExitMenuItem;
#[derive(Component)]
struct ModeSpriteMenuItem;
#[derive(Component)]
struct ClearSelectionButton;
#[derive(Component)]
struct ModeTextLabel;
#[derive(Component)]
struct SelectionCountText;
#[derive(Component)]
struct ImageNameText;
#[derive(Component)]
struct ImageSizeText;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct LeftPanelWidthNode;
#[derive(Component)]
struct RightPanelWidthNode;
#[derive(Component)]
struct BottomPanelHeightNode;
#[derive(Component)]
struct CanvasSurface;
#[derive(Component)]
struct CanvasViewport;
#[derive(Component)]
struct LeftPanelInputRegion;
#[derive(Component)]
struct RightPanelInputRegion;
#[derive(Component)]
struct BottomPanelInputRegion;
#[derive(Component, Clone, Copy)]
struct GridCellButton {
    index: usize,
}
#[derive(Component, Clone, Copy)]
struct SplitterHandle {
    kind: SplitterKind,
}

#[derive(Component, Clone, Copy)]
struct SectionToggleButton {
    section: LeftPanelSection,
}
#[derive(Component, Clone, Copy)]
struct SectionToggleText {
    section: LeftPanelSection,
}
#[derive(Component, Clone, Copy)]
struct SectionBody {
    section: LeftPanelSection,
}
#[derive(Component, Clone, Copy)]
struct GridFieldButton {
    field: GridFieldKind,
}
#[derive(Component, Clone, Copy)]
struct GridFieldText {
    field: GridFieldKind,
}
#[derive(Component)]
struct ActiveAnimationText;
#[derive(Component)]
struct ActiveTrackText;
#[derive(Component)]
struct PlaybackText;
#[derive(Component, Clone, Copy)]
struct DirectionButton {
    direction: Direction,
}
#[derive(Component)]
struct AppendMirroredCopyButton;
#[derive(Component)]
struct ToggleStepFlipButton;
#[derive(Component)]
struct ApplyStepMsButton;
#[derive(Component)]
struct SetPlaybackLoopButton;
#[derive(Component)]
struct SetPlaybackLoopNButton;
#[derive(Component)]
struct SetPlaybackOneShotButton;
#[derive(Component)]
struct ApplyLoopNTimesButton;
#[derive(Component)]
struct ToggleHoldLastButton;
#[derive(Component, Clone, Copy)]
struct AnimationFieldButton {
    field: AnimationFieldKind,
}
#[derive(Component, Clone, Copy)]
struct AnimationFieldText {
    field: AnimationFieldKind,
}
#[derive(Component, Clone)]
struct AnimationCategoryButton {
    category: String,
}
#[derive(Component, Clone)]
struct AnimationCategoryBody {
    category: String,
}
#[derive(Component, Clone)]
struct AnimationCategoryArrowText {
    category: String,
}
#[derive(Component, Clone, Copy)]
struct AnimationClipButton {
    clip_index: usize,
}
#[derive(Component)]
struct PartsStatusText;
#[derive(Component, Clone, Copy)]
struct PartPaletteCurrentText {
    layer: LayerCode,
}
#[derive(Component)]
struct PaletteStatusText;
#[derive(Component)]
struct PaletteCurrentText;
#[derive(Component)]
struct PalettePathText;
#[derive(Component, Clone, Copy)]
struct PartCurrentText {
    layer: LayerCode,
}
#[derive(Component, Clone, Copy)]
struct PartCycleButton {
    layer: LayerCode,
    delta: i8,
}
#[derive(Component, Clone, Copy)]
struct PartPaletteCycleButton {
    layer: LayerCode,
    delta: i8,
}
#[derive(Component, Clone, Copy)]
struct PaletteCycleButton {
    delta: i8,
}
#[derive(Component, Clone, Copy)]
struct PreviewStepButton {
    step_index: usize,
}
#[derive(Component)]
struct PreviewStripContent;
#[derive(Component, Clone, Copy)]
struct DirectionButtonStyleMarker {
    direction: Direction,
}
#[derive(Component, Clone, Copy)]
struct ViewerLayerImageNode {
    layer: LayerCode,
}
#[derive(Component)]
struct ViewerClipText;
#[derive(Component)]
struct ViewerCellText;
#[derive(Component)]
struct ViewerStepText;
#[derive(Component)]
struct ViewerMsText;
#[derive(Component)]
struct ViewerFlipText;
#[derive(Component, Clone, Copy)]
struct ViewerDirectionButton {
    direction: Direction,
}
#[derive(Component, Clone, Copy)]
struct ViewerDirectionButtonStyleMarker {
    direction: Direction,
}
#[derive(Component)]
struct ViewerPlayPauseButton;
#[derive(Component)]
struct ViewerPlayPauseLabel;
#[derive(Component)]
struct ViewerPrevFrameButton;
#[derive(Component)]
struct ViewerNextFrameButton;
#[derive(Component, Clone, Copy)]
struct ViewerSpeedButton {
    speed: f32,
}
#[derive(Component, Clone, Copy)]
struct ViewerSpeedButtonStyleMarker {
    speed: f32,
}
#[derive(Component)]
struct ViewerLoopOverrideButton;
#[derive(Component)]
struct ViewerLoopOverrideLabel;

#[derive(Resource, Default)]
struct ToolbarUiState {
    show_file_menu: bool,
    show_mode_menu: bool,
    request_open_image: bool,
    request_load_project: bool,
    request_save_project: bool,
}

#[derive(Clone, Copy)]
enum LeftPanelSection {
    SpriteSheet,
    GridSettings,
    Parts,
    Palettes,
    Animations,
}

#[derive(Resource, Clone)]
struct LeftPanelSectionsState {
    sprite_sheet_open: bool,
    grid_settings_open: bool,
    parts_open: bool,
    palettes_open: bool,
    animations_open: bool,
}

impl Default for LeftPanelSectionsState {
    fn default() -> Self {
        Self {
            sprite_sheet_open: true,
            grid_settings_open: true,
            parts_open: false,
            palettes_open: false,
            animations_open: false,
        }
    }
}

#[derive(Resource, Default, Clone)]
struct PalettePanelState {
    selected: Option<usize>,
    variant: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct LayerPaletteSelection {
    palette_index: usize,
    variant: usize,
}

#[derive(Resource, Default, Clone)]
struct LayerPaletteState {
    by_layer: HashMap<LayerCode, LayerPaletteSelection>,
}

#[derive(Resource, Default)]
struct PaletteRemapCache {
    remapped_handles: HashMap<String, Handle<Image>>,
    remap_tables: HashMap<String, HashMap<u32, [u8; 4]>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GridFieldKind {
    Rows,
    Columns,
    CellWidth,
    CellHeight,
    OffsetX,
    OffsetY,
}

#[derive(Resource, Default, Clone)]
struct GridFieldDrafts {
    rows: String,
    columns: String,
    cell_width: String,
    cell_height: String,
    offset_x: String,
    offset_y: String,
    active: Option<GridFieldKind>,
    invalid_rows: bool,
    invalid_columns: bool,
    invalid_cell_width: bool,
    invalid_cell_height: bool,
    invalid_offset_x: bool,
    invalid_offset_y: bool,
}

impl GridFieldDrafts {
    fn set_from_grid(&mut self, grid: &GridState) {
        self.rows = grid.rows.to_string();
        self.columns = grid.columns.to_string();
        self.cell_width = grid.cell_width.to_string();
        self.cell_height = grid.cell_height.to_string();
        self.offset_x = grid.offset_x.to_string();
        self.offset_y = grid.offset_y.to_string();
    }

    fn value(&self, field: GridFieldKind) -> &str {
        match field {
            GridFieldKind::Rows => &self.rows,
            GridFieldKind::Columns => &self.columns,
            GridFieldKind::CellWidth => &self.cell_width,
            GridFieldKind::CellHeight => &self.cell_height,
            GridFieldKind::OffsetX => &self.offset_x,
            GridFieldKind::OffsetY => &self.offset_y,
        }
    }

    fn value_mut(&mut self, field: GridFieldKind) -> &mut String {
        match field {
            GridFieldKind::Rows => &mut self.rows,
            GridFieldKind::Columns => &mut self.columns,
            GridFieldKind::CellWidth => &mut self.cell_width,
            GridFieldKind::CellHeight => &mut self.cell_height,
            GridFieldKind::OffsetX => &mut self.offset_x,
            GridFieldKind::OffsetY => &mut self.offset_y,
        }
    }

    fn is_invalid(&self, field: GridFieldKind) -> bool {
        match field {
            GridFieldKind::Rows => self.invalid_rows,
            GridFieldKind::Columns => self.invalid_columns,
            GridFieldKind::CellWidth => self.invalid_cell_width,
            GridFieldKind::CellHeight => self.invalid_cell_height,
            GridFieldKind::OffsetX => self.invalid_offset_x,
            GridFieldKind::OffsetY => self.invalid_offset_y,
        }
    }

    fn set_invalid(&mut self, field: GridFieldKind, invalid: bool) {
        match field {
            GridFieldKind::Rows => self.invalid_rows = invalid,
            GridFieldKind::Columns => self.invalid_columns = invalid,
            GridFieldKind::CellWidth => self.invalid_cell_width = invalid,
            GridFieldKind::CellHeight => self.invalid_cell_height = invalid,
            GridFieldKind::OffsetX => self.invalid_offset_x = invalid,
            GridFieldKind::OffsetY => self.invalid_offset_y = invalid,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimationFieldKind {
    StepMs,
    LoopNTimes,
    HoldLast,
}

#[derive(Resource, Default, Clone)]
struct AnimationFieldDrafts {
    step_ms: String,
    loop_n_times: String,
    hold_last: String,
    active: Option<AnimationFieldKind>,
    invalid_step_ms: bool,
    invalid_loop_n_times: bool,
    invalid_hold_last: bool,
}

impl AnimationFieldDrafts {
    fn set_defaults(&mut self) {
        self.step_ms = "120".to_string();
        self.loop_n_times = "2".to_string();
        self.hold_last = "true".to_string();
    }

    fn value(&self, field: AnimationFieldKind) -> &str {
        match field {
            AnimationFieldKind::StepMs => &self.step_ms,
            AnimationFieldKind::LoopNTimes => &self.loop_n_times,
            AnimationFieldKind::HoldLast => &self.hold_last,
        }
    }

    fn value_mut(&mut self, field: AnimationFieldKind) -> &mut String {
        match field {
            AnimationFieldKind::StepMs => &mut self.step_ms,
            AnimationFieldKind::LoopNTimes => &mut self.loop_n_times,
            AnimationFieldKind::HoldLast => &mut self.hold_last,
        }
    }

    fn is_invalid(&self, field: AnimationFieldKind) -> bool {
        match field {
            AnimationFieldKind::StepMs => self.invalid_step_ms,
            AnimationFieldKind::LoopNTimes => self.invalid_loop_n_times,
            AnimationFieldKind::HoldLast => self.invalid_hold_last,
        }
    }

    fn set_invalid(&mut self, field: AnimationFieldKind, invalid: bool) {
        match field {
            AnimationFieldKind::StepMs => self.invalid_step_ms = invalid,
            AnimationFieldKind::LoopNTimes => self.invalid_loop_n_times = invalid,
            AnimationFieldKind::HoldLast => self.invalid_hold_last = invalid,
        }
    }
}

#[derive(Resource, Clone)]
struct AnimationPanelState {
    selected_category: Option<String>,
    expanded_categories: BTreeSet<String>,
    active_dir: Option<Direction>,
    selected_steps: BTreeSet<usize>,
}

impl Default for AnimationPanelState {
    fn default() -> Self {
        Self {
            selected_category: None,
            expanded_categories: BTreeSet::new(),
            active_dir: None,
            selected_steps: BTreeSet::new(),
        }
    }
}

#[derive(Resource, Default)]
struct PreviewStripUiState {
    dirty: bool,
    dynamic_entities: Vec<Entity>,
}

#[derive(Resource, Default, Clone)]
struct PreviewAtlasState {
    image: Option<Handle<Image>>,
    layout: Option<Handle<TextureAtlasLayout>>,
    columns: u32,
    rows: u32,
    cell_size: UVec2,
    spacing: UVec2,
    offset: UVec2,
}

#[derive(Resource, Clone)]
struct AnimViewerState {
    clip_id: Option<String>,
    dir: Direction,
    is_playing: bool,
    step_index: usize,
    step_elapsed_ms: f32,
    speed: f32,
    loop_override: bool,
    loop_n_remaining: Option<u16>,
}

impl Default for AnimViewerState {
    fn default() -> Self {
        Self {
            clip_id: None,
            dir: Direction::Down,
            is_playing: true,
            step_index: 0,
            step_elapsed_ms: 0.0,
            speed: 1.0,
            loop_override: false,
            loop_n_remaining: None,
        }
    }
}

#[derive(Clone, Copy)]
enum SplitterKind {
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputRegion {
    LeftPanel,
    GridCanvas,
    RightPanel,
    BottomPanel,
}

#[derive(Resource, Default)]
struct InputContext {
    hovered_region: Option<InputRegion>,
    active_capture: Option<InputRegion>,
}
