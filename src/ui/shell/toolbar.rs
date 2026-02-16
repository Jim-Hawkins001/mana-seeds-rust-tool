use super::*;
use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub(super) struct SaveProjectCtx<'w> {
    toolbar_ui: ResMut<'w, ToolbarUiState>,
    anim_state: ResMut<'w, AnimationAuthoringState>,
    paper_doll: Res<'w, PaperDollState>,
    grid: ResMut<'w, GridState>,
    panel_state: Res<'w, AnimationPanelState>,
    grid_fields: ResMut<'w, GridFieldDrafts>,
    animation_fields: ResMut<'w, AnimationFieldDrafts>,
    selection_state: ResMut<'w, SelectionState>,
    canvas_ui: ResMut<'w, CanvasUiState>,
    loaded: ResMut<'w, LoadedImage>,
}

pub(super) struct PendingSaveCtx<'a> {
    grid: &'a mut GridState,
    grid_fields: &'a mut GridFieldDrafts,
    selection_state: &'a mut SelectionState,
    canvas_ui: &'a mut CanvasUiState,
    anim_state: &'a mut AnimationAuthoringState,
    panel_state: &'a AnimationPanelState,
    animation_fields: &'a mut AnimationFieldDrafts,
    loaded: &'a mut LoadedImage,
}

pub(super) fn handle_toolbar_menu_toggles(
    mut toggle_queries: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<FileMenuToggleButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<ModeMenuToggleButton>)>,
    )>,
    mut toolbar_ui: ResMut<ToolbarUiState>,
) {
    for interaction in &mut toggle_queries.p0() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        toolbar_ui.show_file_menu = !toolbar_ui.show_file_menu;
        if toolbar_ui.show_file_menu {
            toolbar_ui.show_mode_menu = false;
        }
    }

    for interaction in &mut toggle_queries.p1() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        toolbar_ui.show_mode_menu = !toolbar_ui.show_mode_menu;
        if toolbar_ui.show_mode_menu {
            toolbar_ui.show_file_menu = false;
        }
    }
}

pub(super) fn handle_toolbar_menu_actions(
    mut menu_queries: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<FileOpenMenuItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<FileLoadProjectMenuItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<FileSaveMenuItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<FileExitMenuItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<ModeAnimationsMenuItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<ModePartsMenuItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<ModeOutfitsMenuItem>)>,
    )>,
    mut toolbar_ui: ResMut<ToolbarUiState>,
    mut mode: ResMut<EditorMode>,
    mut loaded: ResMut<LoadedImage>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    for interaction in &mut menu_queries.p0() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        toolbar_ui.request_open_image = true;
        toolbar_ui.show_file_menu = false;
    }

    for interaction in &mut menu_queries.p1() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        toolbar_ui.request_load_project = true;
        toolbar_ui.show_file_menu = false;
    }

    for interaction in &mut menu_queries.p2() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        toolbar_ui.request_save_project = true;
        toolbar_ui.show_file_menu = false;
    }

    for interaction in &mut menu_queries.p3() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        exit_writer.write(AppExit::Success);
    }

    for interaction in &mut menu_queries.p4() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *mode = EditorMode::Animations;
        loaded.status = Some("Mode: Animations".to_string());
        toolbar_ui.show_mode_menu = false;
    }

    for interaction in &mut menu_queries.p5() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *mode = EditorMode::Parts;
        loaded.status = Some("Mode: Parts".to_string());
        toolbar_ui.show_mode_menu = false;
    }

    for interaction in &mut menu_queries.p6() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *mode = EditorMode::Outfits;
        loaded.status = Some("Mode: Outfits".to_string());
        toolbar_ui.show_mode_menu = false;
    }
}

pub(super) fn sync_toolbar_menu_visibility(
    toolbar_ui: Res<ToolbarUiState>,
    mut menu_nodes: ParamSet<(
        Query<&mut Node, With<FileMenuPanel>>,
        Query<&mut Node, With<ModeMenuPanel>>,
    )>,
) {
    if !toolbar_ui.is_changed() {
        return;
    }

    for mut node in &mut menu_nodes.p0() {
        node.display = if toolbar_ui.show_file_menu {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut menu_nodes.p1() {
        node.display = if toolbar_ui.show_mode_menu {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn handle_open_image_button(
    mut toolbar_ui: ResMut<ToolbarUiState>,
    asset_server: Res<AssetServer>,
    mut loaded: ResMut<LoadedImage>,
    mut selection_state: ResMut<SelectionState>,
    mut canvas_ui: ResMut<CanvasUiState>,
) {
    if !toolbar_ui.request_open_image {
        return;
    }

    toolbar_ui.request_open_image = false;

    let Some(path) = FileDialog::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "tga", "webp"])
        .pick_file()
    else {
        loaded.status = Some("Open image cancelled".to_string());
        return;
    };

    let Some(asset_path) = to_asset_path(&path) else {
        loaded.status = Some("Image must be inside this project's assets/ folder".to_string());
        return;
    };

    let handle = asset_server.load(asset_path.clone());
    let name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());
    loaded.set_pending(asset_path, name, handle);
    selection_state.selected_cells.clear();
    canvas_ui.dirty = true;
}

pub(super) fn handle_load_project_button(
    mut toolbar_ui: ResMut<ToolbarUiState>,
    mut anim_state: ResMut<AnimationAuthoringState>,
    mut paper_doll: ResMut<PaperDollState>,
    mut loaded: ResMut<LoadedImage>,
    mut grid: ResMut<GridState>,
    mut panel_state: ResMut<AnimationPanelState>,
    mut animation_fields: ResMut<AnimationFieldDrafts>,
) {
    if !toolbar_ui.request_load_project {
        return;
    }
    toolbar_ui.request_load_project = false;

    let Some(path) = FileDialog::new().add_filter("RON", &["ron"]).pick_file() else {
        loaded.status = Some("Load animation project cancelled".to_string());
        return;
    };

    match anim_state.load_from_path(&path) {
        Ok(()) => {
            let grid_meta = &anim_state.project.sheet.grid;
            grid.rows = grid_meta.rows.max(1);
            grid.columns = grid_meta.columns.max(1);
            grid.cell_width = grid_meta.cell_w.max(1);
            grid.cell_height = grid_meta.cell_h.max(1);
            grid.offset_x = grid_meta.offset_x;
            grid.offset_y = grid_meta.offset_y;
            panel_state.active_dir = None;
            panel_state.selected_steps.clear();
            panel_state.expanded_categories.clear();
            if let Some(clip) = anim_state.active_clip() {
                panel_state.selected_category = Some(clip.category.clone());
                panel_state
                    .expanded_categories
                    .insert(clip.category.clone());
            }

            let equipped_keys = anim_state.project.layers.equipped_parts.clone();
            if paper_doll.loaded {
                let PaperDollState {
                    catalog, equipped, ..
                } = &mut *paper_doll;
                if equipped_keys.is_empty() {
                    equipped.set_defaults(catalog);
                } else {
                    equipped.apply_equipped_keys(catalog, &equipped_keys);
                }
            } else {
                paper_doll.pending_equipped_keys = Some(equipped_keys);
            }

            animation_fields.set_defaults();
            loaded.status = Some(format!("Loaded animation project: {}", path.display()));
        }
        Err(err) => {
            loaded.status = Some(err);
        }
    }
}

pub(super) fn handle_save_project_button(ctx: SaveProjectCtx) {
    let SaveProjectCtx {
        mut toolbar_ui,
        mut anim_state,
        paper_doll,
        mut grid,
        panel_state,
        mut grid_fields,
        mut animation_fields,
        mut selection_state,
        mut canvas_ui,
        mut loaded,
    } = ctx;

    if !toolbar_ui.request_save_project {
        return;
    }
    toolbar_ui.request_save_project = false;

    if !flush_pending_drafts_for_save(PendingSaveCtx {
        grid: &mut grid,
        grid_fields: &mut grid_fields,
        selection_state: &mut selection_state,
        canvas_ui: &mut canvas_ui,
        anim_state: &mut anim_state,
        panel_state: &panel_state,
        animation_fields: &mut animation_fields,
        loaded: &mut loaded,
    }) {
        return;
    }

    if paper_doll.loaded {
        let equipped_keys = paper_doll.equipped.equipped_part_keys(&paper_doll.catalog);
        anim_state.project.layers.equipped_parts = equipped_keys.clone();
        for key in equipped_keys {
            anim_state
                .project
                .layers
                .mappings
                .entry(key)
                .or_insert(PartMapping::FollowBodyCells);
        }
    }

    anim_state.sync_sheet_meta(&grid, &loaded);

    let path = if let Some(existing) = anim_state.save_path.clone() {
        existing
    } else {
        let Some(chosen) = FileDialog::new()
            .add_filter("RON", &["ron"])
            .set_file_name("anim_project.ron")
            .save_file()
        else {
            loaded.status = Some("Save animation project cancelled".to_string());
            return;
        };
        anim_state.save_path = Some(chosen.clone());
        chosen
    };

    match anim_state.save_to_path(&path) {
        Ok(()) => loaded.status = Some(format!("Saved animation project: {}", path.display())),
        Err(err) => loaded.status = Some(err),
    }
}

pub(super) fn flush_pending_drafts_for_save(ctx: PendingSaveCtx<'_>) -> bool {
    let PendingSaveCtx {
        grid,
        grid_fields,
        selection_state,
        canvas_ui,
        anim_state,
        panel_state,
        animation_fields,
        loaded,
    } = ctx;

    if let Some(active_grid_field) = grid_fields.active {
        commit_grid_field(
            active_grid_field,
            grid_fields,
            grid,
            selection_state,
            loaded,
            canvas_ui,
        );
        if grid_fields.is_invalid(active_grid_field) {
            loaded.status = Some("Cannot save: fix invalid grid field input".to_string());
            return false;
        }
        grid_fields.active = None;
    }

    let apply_step_ms = is_edit_armed(panel_state.active_dir);
    let playback_mode = anim_state.active_clip().map(|clip| clip.playback);
    let playback_is_loop_n = matches!(playback_mode, Some(Playback::LoopN { .. }));
    let playback_is_one_shot = matches!(playback_mode, Some(Playback::OneShot { .. }));

    let step_ms = if apply_step_ms {
        match animation_fields.step_ms.trim().parse::<u16>() {
            Ok(ms) => {
                animation_fields.set_invalid(AnimationFieldKind::StepMs, false);
                ms.max(1)
            }
            Err(_) => {
                animation_fields.set_invalid(AnimationFieldKind::StepMs, true);
                loaded.status = Some("Cannot save: Duration (ms) must be numeric".to_string());
                return false;
            }
        }
    } else {
        animation_fields.set_invalid(AnimationFieldKind::StepMs, false);
        1
    };

    let loop_n_times = if playback_is_loop_n {
        match animation_fields.loop_n_times.trim().parse::<u16>() {
            Ok(times) => {
                animation_fields.set_invalid(AnimationFieldKind::LoopNTimes, false);
                times.max(1)
            }
            Err(_) => {
                animation_fields.set_invalid(AnimationFieldKind::LoopNTimes, true);
                loaded.status = Some("Cannot save: Loop N Times must be numeric".to_string());
                return false;
            }
        }
    } else {
        animation_fields.set_invalid(AnimationFieldKind::LoopNTimes, false);
        1
    };

    let hold_last = if playback_is_one_shot {
        match parse_bool_input(&animation_fields.hold_last) {
            Some(value) => {
                animation_fields.set_invalid(AnimationFieldKind::HoldLast, false);
                value
            }
            None => {
                animation_fields.set_invalid(AnimationFieldKind::HoldLast, true);
                loaded.status = Some("Cannot save: Hold Last Frame must be true/false".to_string());
                return false;
            }
        }
    } else {
        animation_fields.set_invalid(AnimationFieldKind::HoldLast, false);
        false
    };

    if apply_step_ms {
        if let Some(active_dir) = panel_state.active_dir {
            anim_state.active_direction = active_dir;
        }
        let selected_steps: Vec<usize> = panel_state.selected_steps.iter().copied().collect();
        if let Some(track) = anim_state.active_track_mut() {
            if selected_steps.is_empty() {
                for step in &mut track.steps {
                    step.ms = step_ms;
                }
            } else {
                for step_index in selected_steps {
                    if let Some(step) = track.steps.get_mut(step_index) {
                        step.ms = step_ms;
                    }
                }
            }
        }
    }

    if let Some(clip) = anim_state.active_clip_mut() {
        if let Playback::LoopN { times } = &mut clip.playback {
            *times = loop_n_times;
        }
        if let Playback::OneShot {
            hold_last: clip_hold_last,
        } = &mut clip.playback
        {
            *clip_hold_last = hold_last;
        }
    }

    animation_fields.active = None;
    true
}
