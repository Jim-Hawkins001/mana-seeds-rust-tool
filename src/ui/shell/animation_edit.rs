use super::*;
use bevy::ecs::system::SystemParam;

type ButtonPressQuery<'w, 's, T> =
    Query<'w, 's, &'static Interaction, (Changed<Interaction>, With<T>)>;
type ButtonPairPressQuery<'w, 's, T> =
    Query<'w, 's, (&'static Interaction, &'static T), (Changed<Interaction>, With<Button>)>;

#[derive(SystemParam)]
pub(super) struct AnimationNavigationCtx<'w, 's> {
    loaded: ResMut<'w, LoadedImage>,
    selection_state: ResMut<'w, SelectionState>,
    anim_state: ResMut<'w, AnimationAuthoringState>,
    panel_state: ResMut<'w, AnimationPanelState>,
    animation_fields: ResMut<'w, AnimationFieldDrafts>,
    direction_buttons: ButtonPairPressQuery<'w, 's, DirectionButton>,
    append_mirror_buttons: ButtonPressQuery<'w, 's, AppendMirroredCopyButton>,
    toggle_flip_buttons: ButtonPressQuery<'w, 's, ToggleStepFlipButton>,
}

#[derive(SystemParam)]
pub(super) struct AnimationDataCtx<'w, 's> {
    loaded: ResMut<'w, LoadedImage>,
    anim_state: ResMut<'w, AnimationAuthoringState>,
    panel_state: Res<'w, AnimationPanelState>,
    animation_fields: ResMut<'w, AnimationFieldDrafts>,
    apply_step_ms_buttons: ButtonPressQuery<'w, 's, ApplyStepMsButton>,
    set_loop_buttons: ButtonPressQuery<'w, 's, SetPlaybackLoopButton>,
    set_loop_n_buttons: ButtonPressQuery<'w, 's, SetPlaybackLoopNButton>,
    set_one_shot_buttons: ButtonPressQuery<'w, 's, SetPlaybackOneShotButton>,
    apply_loop_n_buttons: ButtonPressQuery<'w, 's, ApplyLoopNTimesButton>,
    apply_hold_last_buttons: ButtonPressQuery<'w, 's, ToggleHoldLastButton>,
}

#[derive(SystemParam)]
pub(super) struct AnimViewerControlsCtx<'w, 's> {
    direction_buttons: ButtonPairPressQuery<'w, 's, ViewerDirectionButton>,
    play_pause_buttons: ButtonPressQuery<'w, 's, ViewerPlayPauseButton>,
    prev_buttons: ButtonPressQuery<'w, 's, ViewerPrevFrameButton>,
    next_buttons: ButtonPressQuery<'w, 's, ViewerNextFrameButton>,
    speed_buttons: ButtonPairPressQuery<'w, 's, ViewerSpeedButton>,
    loop_override_buttons: ButtonPressQuery<'w, 's, ViewerLoopOverrideButton>,
    anim_state: Res<'w, AnimationAuthoringState>,
    viewer: ResMut<'w, AnimViewerState>,
    loaded: ResMut<'w, LoadedImage>,
}

pub(super) fn handle_animation_navigation_buttons(ctx: AnimationNavigationCtx) {
    let AnimationNavigationCtx {
        mut loaded,
        mut selection_state,
        mut anim_state,
        mut panel_state,
        mut animation_fields,
        direction_buttons,
        append_mirror_buttons,
        toggle_flip_buttons,
    } = ctx;

    for (interaction, direction_button) in &direction_buttons {
        if *interaction == Interaction::Pressed {
            if panel_state.active_dir == Some(direction_button.direction) {
                panel_state.active_dir = None;
                panel_state.selected_steps.clear();
                anim_state.active_step = None;
                selection_state.selected_cells.clear();
                loaded.status = Some("Viewing".to_string());
            } else {
                panel_state.active_dir = Some(direction_button.direction);
                panel_state.selected_steps.clear();
                anim_state.active_step = None;
                anim_state.active_direction = direction_button.direction;
                sync_workspace_selection_from_track(
                    &anim_state,
                    &panel_state,
                    &mut selection_state,
                );
                if direction_button.direction == Direction::Left {
                    loaded.status = Some("Preview: Left (derived from Right)".to_string());
                } else if let Some(clip) = anim_state.active_clip() {
                    loaded.status = Some(format!(
                        "Editing: {} / {}",
                        clip.display,
                        direction_button.direction.label()
                    ));
                }
            }
            sync_animation_field_drafts_from_state(
                &anim_state,
                &panel_state,
                &mut animation_fields,
            );
        }
    }

    for interaction in &append_mirror_buttons {
        if *interaction == Interaction::Pressed {
            if !is_edit_armed(panel_state.active_dir) {
                loaded.status = Some("Select Up/Down/Right to arm editing".to_string());
                continue;
            }
            let count = anim_state.append_mirrored_copy();
            loaded.status = Some(format!("Appended {count} mirrored step(s)"));
            sync_workspace_selection_from_track(&anim_state, &panel_state, &mut selection_state);
        }
    }
    for interaction in &toggle_flip_buttons {
        if *interaction == Interaction::Pressed {
            if !is_edit_armed(panel_state.active_dir) {
                loaded.status =
                    Some("Flip is only available in Up/Down/Right editing mode".to_string());
                continue;
            }
            let selected_steps: Vec<usize> = panel_state.selected_steps.iter().copied().collect();
            let Some(track) = anim_state.active_track_mut() else {
                continue;
            };
            if selected_steps.is_empty() {
                loaded.status = Some("Select one or more preview frames first".to_string());
                continue;
            }
            for step_index in selected_steps {
                if let Some(step) = track.steps.get_mut(step_index) {
                    step.flip_x = !step.flip_x;
                }
            }
            sync_animation_field_drafts_from_state(
                &anim_state,
                &panel_state,
                &mut animation_fields,
            );
        }
    }
}

pub(super) fn handle_animation_data_buttons(ctx: AnimationDataCtx) {
    let AnimationDataCtx {
        mut loaded,
        mut anim_state,
        panel_state,
        mut animation_fields,
        apply_step_ms_buttons,
        set_loop_buttons,
        set_loop_n_buttons,
        set_one_shot_buttons,
        apply_loop_n_buttons,
        apply_hold_last_buttons,
    } = ctx;

    for interaction in &apply_step_ms_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if !is_edit_armed(panel_state.active_dir) {
            loaded.status = Some("Select Up/Down/Right to arm editing".to_string());
            continue;
        }
        let Ok(ms) = animation_fields.step_ms.trim().parse::<u16>() else {
            animation_fields.set_invalid(AnimationFieldKind::StepMs, true);
            loaded.status = Some("Step ms must be numeric".to_string());
            continue;
        };
        let applied_ms = ms.max(1);
        let selected_steps: Vec<usize> = panel_state.selected_steps.iter().copied().collect();
        let Some(track) = anim_state.active_track_mut() else {
            continue;
        };
        if selected_steps.is_empty() {
            for step in &mut track.steps {
                step.ms = applied_ms;
            }
        } else {
            for step_index in selected_steps {
                if let Some(step) = track.steps.get_mut(step_index) {
                    step.ms = applied_ms;
                }
            }
        }
        sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
    }

    for interaction in &set_loop_buttons {
        if *interaction == Interaction::Pressed {
            if let Some(clip) = anim_state.active_clip_mut() {
                clip.playback = Playback::Loop;
            }
            sync_animation_field_drafts_from_state(
                &anim_state,
                &panel_state,
                &mut animation_fields,
            );
        }
    }

    for interaction in &set_loop_n_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(times) = animation_fields.loop_n_times.trim().parse::<u16>() else {
            animation_fields.set_invalid(AnimationFieldKind::LoopNTimes, true);
            loaded.status = Some("LoopN times must be numeric".to_string());
            continue;
        };
        if let Some(clip) = anim_state.active_clip_mut() {
            clip.playback = Playback::LoopN {
                times: times.max(1),
            };
        }
        sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
    }

    for interaction in &set_one_shot_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(hold_last) = parse_bool_input(&animation_fields.hold_last) else {
            animation_fields.set_invalid(AnimationFieldKind::HoldLast, true);
            loaded.status = Some("hold_last must be true/false".to_string());
            continue;
        };
        if let Some(clip) = anim_state.active_clip_mut() {
            clip.playback = Playback::OneShot { hold_last };
        }
        sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
    }

    for interaction in &apply_loop_n_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(times) = animation_fields.loop_n_times.trim().parse::<u16>() else {
            animation_fields.set_invalid(AnimationFieldKind::LoopNTimes, true);
            loaded.status = Some("LoopN times must be numeric".to_string());
            continue;
        };
        if let Some(clip) = anim_state.active_clip_mut() {
            clip.playback = Playback::LoopN {
                times: times.max(1),
            };
        }
        sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
    }

    for interaction in &apply_hold_last_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(hold_last) = parse_bool_input(&animation_fields.hold_last) else {
            animation_fields.set_invalid(AnimationFieldKind::HoldLast, true);
            loaded.status = Some("hold_last must be true/false".to_string());
            continue;
        };
        if let Some(clip) = anim_state.active_clip_mut() {
            clip.playback = Playback::OneShot { hold_last };
        }
        sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
    }
}

pub(super) fn handle_animation_tree_clicks(
    mut category_buttons: Query<
        (&Interaction, &AnimationCategoryButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut clip_buttons: Query<
        (&Interaction, &AnimationClipButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut panel_state: ResMut<AnimationPanelState>,
    mut anim_state: ResMut<AnimationAuthoringState>,
    mut selection_state: ResMut<SelectionState>,
    mut animation_fields: ResMut<AnimationFieldDrafts>,
    mut loaded: ResMut<LoadedImage>,
) {
    for (interaction, category_button) in &mut category_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if panel_state
            .expanded_categories
            .contains(&category_button.category)
        {
            panel_state
                .expanded_categories
                .remove(&category_button.category);
        } else {
            panel_state
                .expanded_categories
                .insert(category_button.category.clone());
        }
        panel_state.selected_category = Some(category_button.category.clone());
    }

    for (interaction, clip_button) in &mut clip_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        anim_state.set_active_clip(clip_button.clip_index);
        if let Some(clip) = anim_state.active_clip() {
            panel_state.selected_category = Some(clip.category.clone());
            panel_state
                .expanded_categories
                .insert(clip.category.clone());
            loaded.status = Some(format!("Viewing: {}", clip.display));
        }
        panel_state.active_dir = None;
        panel_state.selected_steps.clear();
        anim_state.active_step = None;
        selection_state.selected_cells.clear();
        sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
    }
}

pub(super) fn handle_part_cycle_buttons(
    mut part_buttons: Query<(&Interaction, &PartCycleButton), (Changed<Interaction>, With<Button>)>,
    mut paper_doll: ResMut<PaperDollState>,
    mut loaded: ResMut<LoadedImage>,
) {
    if !paper_doll.loaded {
        for (interaction, _) in &mut part_buttons {
            if *interaction == Interaction::Pressed {
                loaded.status = Some("Parts catalog is still loading".to_string());
            }
        }
        return;
    }

    let PaperDollState {
        catalog,
        equipped,
        image_handles,
        ..
    } = &mut *paper_doll;

    for (interaction, button) in &mut part_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let layer_indices = catalog.layer_indices(button.layer);
        if layer_indices.is_empty() {
            loaded.status = Some(format!("No parts found for {}", button.layer.as_str()));
            continue;
        }

        let current_index = equipped.by_layer.get(&button.layer).copied();
        let current_pos = current_index.and_then(|index| {
            layer_indices
                .iter()
                .position(|candidate| *candidate == index)
        });
        let next_pos = if button.delta >= 0 {
            match current_pos {
                Some(pos) if pos + 1 < layer_indices.len() => Some(pos + 1),
                Some(_) => None,
                None => Some(0),
            }
        } else {
            match current_pos {
                Some(pos) if pos > 0 => Some(pos - 1),
                Some(_) => None,
                None => Some(layer_indices.len() - 1),
            }
        };
        if let Some(next_pos) = next_pos {
            let next_index = layer_indices[next_pos];
            equipped.equip_by_index(catalog, next_index);

            if let Some(part) = catalog.parts.get(next_index) {
                if let Some(handle) = image_handles.get(&part.part_key).cloned() {
                    let name = Path::new(&part.image_path)
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| part.part_id.name.clone());
                    loaded.set_pending(part.image_path.clone(), name, handle);
                }
                loaded.status = Some(format!(
                    "Equipped {}: {}",
                    button.layer.as_str(),
                    format_part_short(part)
                ));
            }
        } else {
            equipped.by_layer.remove(&button.layer);
            loaded.status = Some(format!("Unequipped {}", button.layer.as_str()));
        }
    }
}

pub(super) fn handle_part_palette_cycle_buttons(
    mut palette_buttons: Query<
        (&Interaction, &PartPaletteCycleButton),
        (Changed<Interaction>, With<Button>),
    >,
    paper_doll: Res<PaperDollState>,
    palette_images: Res<Assets<Image>>,
    palette_panel: Res<PalettePanelState>,
    mut layer_palettes: ResMut<LayerPaletteState>,
    mut loaded: ResMut<LoadedImage>,
) {
    if !paper_doll.loaded {
        for (interaction, _) in &mut palette_buttons {
            if *interaction == Interaction::Pressed {
                loaded.status = Some("Palette catalog is still loading".to_string());
            }
        }
        return;
    }

    let palette_count = paper_doll.palette_catalog.palettes.len();
    if palette_count == 0 {
        for (interaction, _) in &mut palette_buttons {
            if *interaction == Interaction::Pressed {
                loaded.status = Some("No palettes discovered".to_string());
            }
        }
        return;
    }

    let global = global_palette_selection(&palette_panel, palette_count);

    for (interaction, button) in &mut palette_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let current = layer_palettes
            .by_layer
            .get(&button.layer)
            .copied()
            .or(global)
            .unwrap_or(LayerPaletteSelection {
                palette_index: 0,
                variant: 0,
            });
        let current_variant_count =
            palette_variant_count(&paper_doll, &palette_images, current.palette_index);
        let current_variant = current.variant.min(current_variant_count.saturating_sub(1));
        let (next_palette, next_variant) = if button.delta >= 0 {
            if current_variant + 1 < current_variant_count {
                (current.palette_index, current_variant + 1)
            } else {
                ((current.palette_index + 1) % palette_count, 0)
            }
        } else if current_variant > 0 {
            (current.palette_index, current_variant - 1)
        } else {
            let prev_palette = (current.palette_index + palette_count - 1) % palette_count;
            let prev_variant_count =
                palette_variant_count(&paper_doll, &palette_images, prev_palette);
            (prev_palette, prev_variant_count.saturating_sub(1))
        };

        let next_selection = LayerPaletteSelection {
            palette_index: next_palette,
            variant: next_variant,
        };
        if Some(next_selection) == global {
            layer_palettes.by_layer.remove(&button.layer);
        } else {
            layer_palettes.by_layer.insert(button.layer, next_selection);
        }

        if let Some(palette) = paper_doll.palette_catalog.palettes.get(next_palette) {
            let name = Path::new(&palette.image_path)
                .file_name()
                .map(|file_name| file_name.to_string_lossy().to_string())
                .unwrap_or_else(|| palette.palette_key.clone());
            loaded.status = Some(format!(
                "Layer {} color: {} ({}/{})",
                button.layer.as_str(),
                name,
                next_variant + 1,
                palette_variant_count(&paper_doll, &palette_images, next_palette)
            ));
        }
    }
}

pub(super) fn handle_palette_cycle_buttons(
    mut palette_buttons: Query<
        (&Interaction, &PaletteCycleButton),
        (Changed<Interaction>, With<Button>),
    >,
    paper_doll: Res<PaperDollState>,
    palette_images: Res<Assets<Image>>,
    mut panel_state: ResMut<PalettePanelState>,
    mut loaded: ResMut<LoadedImage>,
) {
    if !paper_doll.loaded {
        for (interaction, _) in &mut palette_buttons {
            if *interaction == Interaction::Pressed {
                loaded.status = Some("Palette catalog is still loading".to_string());
            }
        }
        return;
    }

    let count = paper_doll.palette_catalog.palettes.len();
    if count == 0 {
        for (interaction, _) in &mut palette_buttons {
            if *interaction == Interaction::Pressed {
                loaded.status = Some("No palettes discovered".to_string());
            }
        }
        return;
    }

    for (interaction, button) in &mut palette_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let current_palette = panel_state.selected.unwrap_or(0).min(count - 1);
        let current_variant_count =
            palette_variant_count(&paper_doll, &palette_images, current_palette);
        let current_variant = panel_state
            .variant
            .min(current_variant_count.saturating_sub(1));

        let (next_palette, next_variant) = if button.delta >= 0 {
            if current_variant + 1 < current_variant_count {
                (current_palette, current_variant + 1)
            } else {
                ((current_palette + 1) % count, 0)
            }
        } else {
            if current_variant > 0 {
                (current_palette, current_variant - 1)
            } else {
                let prev_palette = (current_palette + count - 1) % count;
                let prev_variant_count =
                    palette_variant_count(&paper_doll, &palette_images, prev_palette);
                (prev_palette, prev_variant_count.saturating_sub(1))
            }
        };
        panel_state.selected = Some(next_palette);
        panel_state.variant = next_variant;

        if let Some(palette) = paper_doll.palette_catalog.palettes.get(next_palette) {
            let name = Path::new(&palette.image_path)
                .file_name()
                .map(|file_name| file_name.to_string_lossy().to_string())
                .unwrap_or_else(|| palette.palette_key.clone());
            loaded.status = Some(format!(
                "Palette selected: {name} ({}/{})",
                next_variant + 1,
                palette_variant_count(&paper_doll, &palette_images, next_palette)
            ));
        }
    }
}

pub(super) fn handle_preview_step_buttons(
    mut step_buttons: Query<
        (&Interaction, &PreviewStepButton),
        (Changed<Interaction>, With<Button>),
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_state: ResMut<AnimationPanelState>,
    mut anim_state: ResMut<AnimationAuthoringState>,
    mut animation_fields: ResMut<AnimationFieldDrafts>,
) {
    if panel_state.active_dir.is_none() {
        return;
    }
    let track_len = anim_state
        .active_track()
        .map(|track| track.steps.len())
        .unwrap_or(0);
    if track_len == 0 {
        panel_state.selected_steps.clear();
        anim_state.active_step = None;
        return;
    }

    let multi_select = keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight)
        || keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    for (interaction, step_button) in &mut step_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if step_button.step_index >= track_len {
            continue;
        }
        if multi_select {
            if !panel_state.selected_steps.insert(step_button.step_index) {
                panel_state.selected_steps.remove(&step_button.step_index);
            }
        } else {
            panel_state.selected_steps.clear();
            panel_state.selected_steps.insert(step_button.step_index);
        }
    }

    anim_state.active_step = panel_state.selected_steps.iter().next().copied();
    sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);
}

pub(super) fn sync_parts_panel_text(
    paper_doll: Res<PaperDollState>,
    palette_images: Res<Assets<Image>>,
    palette_panel: Res<PalettePanelState>,
    layer_palettes: Res<LayerPaletteState>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<PartsStatusText>>,
        Query<(&PartCurrentText, &mut Text)>,
        Query<(&PartPaletteCurrentText, &mut Text)>,
    )>,
) {
    if !paper_doll.is_changed()
        && !palette_images.is_changed()
        && !palette_panel.is_changed()
        && !layer_palettes.is_changed()
    {
        return;
    }

    let status = if !paper_doll.loaded {
        "Parts: scanning catalog...".to_string()
    } else if paper_doll.load_errors.is_empty() {
        format!("Parts: {} discovered", paper_doll.catalog.parts.len())
    } else {
        format!(
            "Parts: {} discovered ({} skipped)",
            paper_doll.catalog.parts.len(),
            paper_doll.load_errors.len()
        )
    };

    for mut text in &mut text_sets.p0() {
        *text = Text::new(status.clone());
    }

    for (marker, mut text) in &mut text_sets.p1() {
        let layer_indices = paper_doll.catalog.layer_indices(marker.layer);
        if layer_indices.is_empty() {
            *text = Text::new("(none)");
            continue;
        }
        let label = paper_doll
            .equipped
            .by_layer
            .get(&marker.layer)
            .and_then(|index| paper_doll.catalog.parts.get(*index))
            .map(format_part_short)
            .unwrap_or_else(|| "(none)".to_string());
        *text = Text::new(label);
    }

    let palette_count = paper_doll.palette_catalog.palettes.len();
    let global = global_palette_selection(&palette_panel, palette_count);
    for (marker, mut text) in &mut text_sets.p2() {
        let Some((selection, is_local)) =
            effective_layer_palette_selection(marker.layer, &layer_palettes, global)
        else {
            *text = Text::new("(none)");
            continue;
        };
        let Some(palette) = paper_doll
            .palette_catalog
            .palettes
            .get(selection.palette_index)
        else {
            *text = Text::new("(none)");
            continue;
        };
        let variant_count =
            palette_variant_count(&paper_doll, &palette_images, selection.palette_index);
        let variant = selection.variant.min(variant_count.saturating_sub(1));
        let name = Path::new(&palette.image_path)
            .file_name()
            .map(|file_name| file_name.to_string_lossy().to_string())
            .unwrap_or_else(|| palette.palette_key.clone());
        let scope = if is_local { "L" } else { "G" };
        *text = Text::new(format!("{scope}: {name} {}/{}", variant + 1, variant_count));
    }
}

pub(super) fn sync_palette_panel_text(
    paper_doll: Res<PaperDollState>,
    palette_images: Res<Assets<Image>>,
    mut panel_state: ResMut<PalettePanelState>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<PaletteStatusText>>,
        Query<&mut Text, With<PaletteCurrentText>>,
        Query<&mut Text, With<PalettePathText>>,
    )>,
) {
    if !paper_doll.is_changed() && !palette_images.is_changed() && !panel_state.is_changed() {
        return;
    }

    if !paper_doll.loaded {
        for mut text in &mut text_sets.p0() {
            *text = Text::new("Palettes: scanning catalog...");
        }
        for mut text in &mut text_sets.p1() {
            *text = Text::new("Current: (none)");
        }
        for mut text in &mut text_sets.p2() {
            *text = Text::new("Path: -");
        }
        return;
    }

    let count = paper_doll.palette_catalog.palettes.len();
    if count == 0 {
        panel_state.selected = None;
        for mut text in &mut text_sets.p0() {
            *text = Text::new("Palettes: 0 discovered");
        }
        for mut text in &mut text_sets.p1() {
            *text = Text::new("Current: (none)");
        }
        for mut text in &mut text_sets.p2() {
            *text = Text::new("Path: -");
        }
        return;
    }

    let selected = panel_state.selected.unwrap_or(0).min(count - 1);
    let variant_count = palette_variant_count(&paper_doll, &palette_images, selected);
    let variant = panel_state.variant.min(variant_count.saturating_sub(1));
    if panel_state.selected != Some(selected) {
        panel_state.selected = Some(selected);
    }
    if panel_state.variant != variant {
        panel_state.variant = variant;
    }
    let Some(palette) = paper_doll.palette_catalog.palettes.get(selected) else {
        return;
    };
    let name = Path::new(&palette.image_path)
        .file_name()
        .map(|file_name| file_name.to_string_lossy().to_string())
        .unwrap_or_else(|| palette.palette_key.clone());

    for mut text in &mut text_sets.p0() {
        *text = Text::new(format!(
            "Palettes: {} discovered | Variant: {}/{}",
            count,
            variant + 1,
            variant_count
        ));
    }
    for mut text in &mut text_sets.p1() {
        *text = Text::new(format!(
            "Current: {} ({}/{})",
            name,
            variant + 1,
            variant_count
        ));
    }
    for mut text in &mut text_sets.p2() {
        *text = Text::new(format!("Path: {}", palette.image_path));
    }
}

pub(super) fn palette_variant_count(
    paper_doll: &PaperDollState,
    palette_images: &Assets<Image>,
    palette_index: usize,
) -> usize {
    let Some(palette) = paper_doll.palette_catalog.palettes.get(palette_index) else {
        return 1;
    };
    let Some(palette_handle) = paper_doll.palette_image_handles.get(&palette.palette_key) else {
        return canonical_variant_count(paper_doll, palette_images);
    };
    let Some(palette_image) = palette_images.get(palette_handle) else {
        return canonical_variant_count(paper_doll, palette_images);
    };
    let count = ((palette_image.height() as usize) / 2).max(1);
    if count > 1 {
        count
    } else {
        canonical_variant_count(paper_doll, palette_images)
    }
}

fn canonical_variant_count(paper_doll: &PaperDollState, palette_images: &Assets<Image>) -> usize {
    for (key, handle) in &paper_doll.palette_image_handles {
        let normalized = key.replace('\\', "/").to_ascii_lowercase();
        if normalized.ends_with("palettes/mana seed 3-color ramps")
            && let Some(image) = palette_images.get(handle)
        {
            return ((image.height() as usize) / 2).max(1);
        }
    }
    1
}

pub(super) fn global_palette_selection(
    palette_panel: &PalettePanelState,
    palette_count: usize,
) -> Option<LayerPaletteSelection> {
    if palette_count == 0 {
        return None;
    }
    Some(LayerPaletteSelection {
        palette_index: palette_panel.selected?.min(palette_count - 1),
        variant: palette_panel.variant,
    })
}

pub(super) fn effective_layer_palette_selection(
    layer: LayerCode,
    layer_palettes: &LayerPaletteState,
    global: Option<LayerPaletteSelection>,
) -> Option<(LayerPaletteSelection, bool)> {
    if let Some(local) = layer_palettes.by_layer.get(&layer).copied() {
        Some((local, true))
    } else {
        global.map(|selection| (selection, false))
    }
}

pub(super) fn handle_anim_viewer_controls(ctx: AnimViewerControlsCtx) {
    let AnimViewerControlsCtx {
        mut direction_buttons,
        play_pause_buttons,
        prev_buttons,
        next_buttons,
        speed_buttons,
        loop_override_buttons,
        anim_state,
        mut viewer,
        mut loaded,
    } = ctx;

    let left_enabled = anim_state
        .active_clip()
        .is_some_and(viewer_left_direction_available);

    for (interaction, direction_button) in &mut direction_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if direction_button.direction == Direction::Left && !left_enabled {
            loaded.status =
                Some("Viewer: Left is unavailable (no Right track to mirror)".to_string());
            continue;
        }
        viewer.dir = direction_button.direction;
        reset_viewer_playback(&mut viewer);
        loaded.status = Some(format!(
            "Viewer direction: {}",
            direction_button.direction.label()
        ));
    }

    for interaction in &play_pause_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        viewer.is_playing = !viewer.is_playing;
        loaded.status = Some(if viewer.is_playing {
            "Viewer: Playing".to_string()
        } else {
            "Viewer: Paused".to_string()
        });
    }

    for interaction in &prev_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if viewer.is_playing {
            loaded.status = Some("Viewer: Pause playback to step manually".to_string());
            continue;
        }
        let Some(clip) = anim_state.active_clip() else {
            continue;
        };
        let Some(resolved) = resolve_viewer_track(clip, viewer.dir) else {
            continue;
        };
        if resolved.track.steps.is_empty() {
            continue;
        }
        viewer.step_index = viewer.step_index.saturating_sub(1);
        viewer.step_elapsed_ms = 0.0;
    }

    for interaction in &next_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if viewer.is_playing {
            loaded.status = Some("Viewer: Pause playback to step manually".to_string());
            continue;
        }
        let Some(clip) = anim_state.active_clip() else {
            continue;
        };
        let Some(resolved) = resolve_viewer_track(clip, viewer.dir) else {
            continue;
        };
        let len = resolved.track.steps.len();
        if len == 0 {
            continue;
        }
        viewer.step_index = (viewer.step_index + 1).min(len - 1);
        viewer.step_elapsed_ms = 0.0;
    }

    for (interaction, speed_button) in &speed_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        viewer.speed = speed_button.speed.clamp(0.1, 4.0);
        loaded.status = Some(format!("Viewer speed: {:.1}x", viewer.speed));
    }

    for interaction in &loop_override_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        viewer.loop_override = !viewer.loop_override;
        viewer.loop_n_remaining = None;
        loaded.status = Some(if viewer.loop_override {
            "Viewer loop override: ON".to_string()
        } else {
            "Viewer loop override: OFF".to_string()
        });
    }
}

pub(super) fn handle_animation_field_focus(
    mut fields: Query<(&Interaction, &AnimationFieldButton), (Changed<Interaction>, With<Button>)>,
    mut drafts: ResMut<AnimationFieldDrafts>,
) {
    for (interaction, field) in &mut fields {
        if *interaction == Interaction::Pressed {
            drafts.active = Some(field.field);
        }
    }
}

pub(super) fn handle_animation_field_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut drafts: ResMut<AnimationFieldDrafts>,
    mut loaded: ResMut<LoadedImage>,
) {
    let Some(active) = drafts.active else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(ch) => {
                let filtered = match active {
                    AnimationFieldKind::HoldLast => ch
                        .chars()
                        .filter(|c| c.is_ascii_alphabetic())
                        .collect::<String>(),
                    _ => ch
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>(),
                };
                if filtered.is_empty() {
                    continue;
                }
                drafts.value_mut(active).push_str(&filtered);
                drafts.set_invalid(active, false);
            }
            Key::Backspace => {
                drafts.value_mut(active).pop();
                drafts.set_invalid(active, false);
            }
            Key::Enter => {
                commit_animation_field(active, &mut drafts, &mut loaded);
                drafts.active = None;
            }
            _ => {}
        }
    }
}

pub(super) fn commit_animation_field_on_blur(
    mouse: Res<ButtonInput<MouseButton>>,
    field_hover: Query<&Interaction, With<AnimationFieldButton>>,
    mut drafts: ResMut<AnimationFieldDrafts>,
    mut loaded: ResMut<LoadedImage>,
) {
    let Some(active) = drafts.active else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let hovered_field = field_hover.iter().any(|interaction| {
        *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
    });
    if hovered_field {
        return;
    }

    commit_animation_field(active, &mut drafts, &mut loaded);
    drafts.active = None;
}

pub(super) fn commit_animation_field(
    field: AnimationFieldKind,
    drafts: &mut AnimationFieldDrafts,
    loaded: &mut LoadedImage,
) {
    match field {
        AnimationFieldKind::StepMs => {
            if drafts.step_ms.trim().parse::<u16>().is_err() {
                drafts.set_invalid(field, true);
                loaded.status = Some("Step ms must be numeric".to_string());
                return;
            }
            drafts.set_invalid(field, false);
        }
        AnimationFieldKind::LoopNTimes => {
            if drafts.loop_n_times.trim().parse::<u16>().is_err() {
                drafts.set_invalid(field, true);
                loaded.status = Some("LoopN times must be numeric".to_string());
                return;
            }
            drafts.set_invalid(field, false);
        }
        AnimationFieldKind::HoldLast => {
            if parse_bool_input(&drafts.hold_last).is_none() {
                drafts.set_invalid(field, true);
                loaded.status = Some("hold_last must be true/false".to_string());
                return;
            }
            drafts.set_invalid(field, false);
        }
    }
}

pub(super) fn to_asset_path(path: &Path) -> Option<String> {
    let canonical_path = std::fs::canonicalize(path).ok()?;
    for root in candidate_assets_roots() {
        let Ok(canonical_root) = std::fs::canonicalize(root) else {
            continue;
        };
        if let Ok(relative) = canonical_path.strip_prefix(canonical_root) {
            return Some(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    None
}

pub(super) fn candidate_assets_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let Ok(cwd) = std::env::current_dir() else {
        return roots;
    };
    roots.push(cwd.join("assets"));
    roots.push(cwd.join("asset_hander").join("assets"));
    for ancestor in cwd.ancestors() {
        roots.push(ancestor.join("assets"));
    }
    roots
}

pub(super) fn handle_clear_selection_button(
    mut clear_buttons: Query<&Interaction, (Changed<Interaction>, With<ClearSelectionButton>)>,
    mut selection_state: ResMut<SelectionState>,
) {
    for interaction in &mut clear_buttons {
        if *interaction == Interaction::Pressed {
            selection_state.selected_cells.clear();
        }
    }
}

pub(super) fn handle_left_panel_section_toggles(
    mut toggles: Query<(&Interaction, &SectionToggleButton), (Changed<Interaction>, With<Button>)>,
    mut sections: ResMut<LeftPanelSectionsState>,
) {
    for (interaction, toggle) in &mut toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let open = !section_is_open(&sections, toggle.section);
        set_section_open(&mut sections, toggle.section, open);
        if open {
            match toggle.section {
                LeftPanelSection::GridSettings => {
                    sections.parts_open = false;
                    sections.palettes_open = false;
                    sections.animations_open = false;
                }
                LeftPanelSection::Parts => {
                    sections.grid_settings_open = false;
                    sections.palettes_open = false;
                    sections.animations_open = false;
                }
                LeftPanelSection::Palettes => {
                    sections.grid_settings_open = false;
                    sections.parts_open = false;
                    sections.animations_open = false;
                }
                LeftPanelSection::Animations => {
                    sections.grid_settings_open = false;
                    sections.parts_open = false;
                    sections.palettes_open = false;
                }
                LeftPanelSection::SpriteSheet => {}
            }
        }
    }
}

pub(super) fn sync_left_panel_sections(
    sections: Res<LeftPanelSectionsState>,
    mut bodies: Query<(&SectionBody, &mut Node)>,
    mut icons: Query<(&SectionToggleText, &mut Text)>,
) {
    if !sections.is_changed() {
        return;
    }
    for (body, mut node) in &mut bodies {
        node.display = if section_is_open(&sections, body.section) {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (icon, mut text) in &mut icons {
        let arrow = if section_is_open(&sections, icon.section) {
            "▾"
        } else {
            "▸"
        };
        *text = Text::new(arrow);
    }
}

pub(super) fn sync_mode_section_defaults(
    mode: Res<EditorMode>,
    mut left_sections: ResMut<LeftPanelSectionsState>,
    mut right_sections: ResMut<RightPanelSectionsState>,
) {
    if !mode.is_changed() {
        return;
    }

    match *mode {
        EditorMode::Animations => {
            left_sections.sprite_sheet_open = true;
            left_sections.grid_settings_open = false;
            left_sections.parts_open = false;
            left_sections.palettes_open = false;
            left_sections.animations_open = true;
            right_sections.playback_open = true;
            right_sections.outfits_open = false;
        }
        EditorMode::Parts => {
            left_sections.sprite_sheet_open = true;
            left_sections.grid_settings_open = false;
            left_sections.parts_open = true;
            left_sections.palettes_open = true;
            left_sections.animations_open = false;
            right_sections.playback_open = true;
            right_sections.outfits_open = false;
        }
        EditorMode::Outfits => {
            left_sections.sprite_sheet_open = false;
            left_sections.grid_settings_open = false;
            left_sections.parts_open = true;
            left_sections.palettes_open = true;
            left_sections.animations_open = false;
            right_sections.playback_open = false;
            right_sections.outfits_open = true;
        }
    }
}

pub(super) fn handle_right_panel_playback_section_toggle(
    mut toggles: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<RightPanelPlaybackSectionToggleButton>,
        ),
    >,
    mut sections: ResMut<RightPanelSectionsState>,
) {
    for interaction in &mut toggles {
        if *interaction == Interaction::Pressed {
            sections.playback_open = !sections.playback_open;
        }
    }
}

pub(super) fn sync_right_panel_playback_section(
    sections: Res<RightPanelSectionsState>,
    mut bodies: Query<&mut Node, With<RightPanelPlaybackSectionBody>>,
    mut icons: Query<&mut Text, With<RightPanelPlaybackSectionToggleText>>,
) {
    if !sections.is_changed() {
        return;
    }

    let display = if sections.playback_open {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut bodies {
        node.display = display;
    }

    let arrow = if sections.playback_open { "v" } else { ">" };
    for mut text in &mut icons {
        *text = Text::new(arrow);
    }
}

pub(super) fn handle_right_panel_outfit_section_toggle(
    mut toggles: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<RightPanelOutfitSectionToggleButton>,
        ),
    >,
    mut sections: ResMut<RightPanelSectionsState>,
) {
    for interaction in &mut toggles {
        if *interaction == Interaction::Pressed {
            sections.outfits_open = !sections.outfits_open;
        }
    }
}

pub(super) fn sync_right_panel_outfit_section(
    sections: Res<RightPanelSectionsState>,
    mut bodies: Query<&mut Node, With<RightPanelOutfitSectionBody>>,
    mut icons: Query<&mut Text, With<RightPanelOutfitSectionToggleText>>,
) {
    if !sections.is_changed() {
        return;
    }

    let display = if sections.outfits_open {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut bodies {
        node.display = display;
    }

    let arrow = if sections.outfits_open { "v" } else { ">" };
    for mut text in &mut icons {
        *text = Text::new(arrow);
    }
}

pub(super) fn handle_outfit_action_buttons(
    mut add_buttons: Query<&Interaction, (Changed<Interaction>, With<AddOutfitButton>)>,
    mut delete_buttons: Query<&Interaction, (Changed<Interaction>, With<DeleteOutfitButton>)>,
    mut save_buttons: Query<&Interaction, (Changed<Interaction>, With<SaveOutfitChangesButton>)>,
    mut add_tag_buttons: Query<&Interaction, (Changed<Interaction>, With<AddOutfitTagButton>)>,
    mut outfits: ResMut<OutfitDbState>,
    mut drafts: ResMut<OutfitFieldDrafts>,
    mut paper_doll: ResMut<PaperDollState>,
    mut palette_panel: ResMut<PalettePanelState>,
    mut layer_palettes: ResMut<LayerPaletteState>,
    mut loaded: ResMut<LoadedImage>,
) {
    for interaction in &mut add_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let new_id = next_outfit_id(&outfits);
        let display_name = new_id.replace('_', " ");
        let outfit = snapshot_outfit_from_preview(
            &paper_doll,
            &palette_panel,
            &layer_palettes,
            new_id.clone(),
            display_name,
            Vec::new(),
        );
        outfits.db.outfits.push(outfit.clone());
        outfits.selected = Some(outfits.db.outfits.len() - 1);
        outfits.dirty = true;
        drafts.set_from_outfit(&outfit);
        loaded.status = Some(format!("Added outfit: {}", outfit.outfit_id));
    }

    for interaction in &mut delete_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(selected) = outfits.selected else {
            loaded.status = Some("Outfits: no selected outfit to delete".to_string());
            continue;
        };
        if selected >= outfits.db.outfits.len() {
            outfits.selected = None;
            drafts.clear_for_none_selected();
            continue;
        }
        let deleted = outfits.db.outfits.remove(selected);
        outfits.selected = if outfits.db.outfits.is_empty() {
            None
        } else {
            Some(selected.min(outfits.db.outfits.len() - 1))
        };
        if let Some(new_selected) = outfits.selected {
            if let Some(outfit) = outfits.db.outfits.get(new_selected) {
                drafts.set_from_outfit(outfit);
                apply_outfit_to_preview(
                    outfit,
                    &mut paper_doll,
                    &mut palette_panel,
                    &mut layer_palettes,
                );
            }
        } else {
            drafts.clear_for_none_selected();
        }
        outfits.dirty = true;
        loaded.status = Some(format!("Deleted outfit: {}", deleted.outfit_id));
    }

    for interaction in &mut add_tag_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(selected) = outfits.selected else {
            loaded.status = Some("Outfits: select an outfit before adding tags".to_string());
            continue;
        };
        let Some(outfit) = outfits.db.outfits.get_mut(selected) else {
            continue;
        };
        let tag = normalize_tag(&drafts.tag_input);
        if tag.is_empty() {
            loaded.status = Some("Outfits: tag input is empty".to_string());
            continue;
        }
        if !outfit.tags.iter().any(|existing| existing == &tag) {
            outfit.tags.push(tag.clone());
            outfits.dirty = true;
            loaded.status = Some(format!("Outfits: added tag '{tag}'"));
        }
        drafts.tag_input.clear();
        drafts.active = None;
    }

    for interaction in &mut save_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(selected) = outfits.selected else {
            loaded.status = Some("Outfits: no selected outfit to save".to_string());
            continue;
        };
        let Some(current) = outfits.db.outfits.get(selected).cloned() else {
            continue;
        };
        let outfit_id = sanitize_outfit_id(&drafts.outfit_id);
        if outfit_id.is_empty() {
            drafts.invalid_outfit_id = true;
            loaded.status = Some("Outfit ID must contain letters/digits/underscores".to_string());
            continue;
        }
        if outfits
            .db
            .outfits
            .iter()
            .enumerate()
            .any(|(index, outfit)| index != selected && outfit.outfit_id == outfit_id)
        {
            drafts.invalid_outfit_id = true;
            loaded.status = Some("Outfit ID must be unique".to_string());
            continue;
        }
        drafts.invalid_outfit_id = false;

        let display_name = if drafts.display_name.trim().is_empty() {
            current.display_name
        } else {
            drafts.display_name.trim().to_string()
        };
        let snapshot = snapshot_outfit_from_preview(
            &paper_doll,
            &palette_panel,
            &layer_palettes,
            outfit_id.clone(),
            display_name,
            current.tags,
        );
        if let Some(slot) = outfits.db.outfits.get_mut(selected) {
            *slot = snapshot;
        }
        drafts.outfit_id = outfit_id;
        outfits.dirty = true;

        match outfits.save() {
            Ok(()) => {
                loaded.status = Some(format!("Saved outfits db: {}", outfits.path.display()));
            }
            Err(err) => {
                loaded.status = Some(err);
            }
        }
    }
}

pub(super) fn handle_outfit_list_clicks(
    mut list_buttons: Query<
        (&Interaction, &OutfitListItemButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut outfits: ResMut<OutfitDbState>,
    mut drafts: ResMut<OutfitFieldDrafts>,
    mut paper_doll: ResMut<PaperDollState>,
    mut palette_panel: ResMut<PalettePanelState>,
    mut layer_palettes: ResMut<LayerPaletteState>,
    mut loaded: ResMut<LoadedImage>,
) {
    for (interaction, item) in &mut list_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(outfit) = outfits.db.outfits.get(item.index).cloned() else {
            continue;
        };
        outfits.selected = Some(item.index);
        drafts.set_from_outfit(&outfit);
        apply_outfit_to_preview(
            &outfit,
            &mut paper_doll,
            &mut palette_panel,
            &mut layer_palettes,
        );
        loaded.status = Some(format!("Loaded outfit: {}", outfit.outfit_id));
    }
}

pub(super) fn handle_outfit_tag_remove_buttons(
    mut remove_buttons: Query<
        (&Interaction, &OutfitRemoveTagButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut outfits: ResMut<OutfitDbState>,
    mut loaded: ResMut<LoadedImage>,
) {
    for (interaction, remove) in &mut remove_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(selected) = outfits.selected else {
            continue;
        };
        let Some(outfit) = outfits.db.outfits.get_mut(selected) else {
            continue;
        };
        if remove.tag_index >= outfit.tags.len() {
            continue;
        }
        let removed = outfit.tags.remove(remove.tag_index);
        outfits.dirty = true;
        loaded.status = Some(format!("Outfits: removed tag '{removed}'"));
        break;
    }
}

pub(super) fn handle_outfit_field_focus(
    mut fields: Query<(&Interaction, &OutfitFieldButton), (Changed<Interaction>, With<Button>)>,
    mut drafts: ResMut<OutfitFieldDrafts>,
) {
    for (interaction, field) in &mut fields {
        if *interaction == Interaction::Pressed {
            drafts.active = Some(field.field);
        }
    }
}

pub(super) fn handle_outfit_field_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut drafts: ResMut<OutfitFieldDrafts>,
    mut outfits: ResMut<OutfitDbState>,
    mut loaded: ResMut<LoadedImage>,
) {
    let Some(active) = drafts.active else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(ch) => {
                let filtered = filter_outfit_input(active, ch);
                if filtered.is_empty() {
                    continue;
                }
                drafts.value_mut(active).push_str(&filtered);
                if active == OutfitFieldKind::OutfitId {
                    drafts.invalid_outfit_id = false;
                }
            }
            Key::Backspace => {
                drafts.value_mut(active).pop();
                if active == OutfitFieldKind::OutfitId {
                    drafts.invalid_outfit_id = false;
                }
            }
            Key::Enter => {
                commit_outfit_field(active, &mut drafts, &mut outfits, &mut loaded);
                drafts.active = None;
            }
            _ => {}
        }
    }
}

pub(super) fn commit_outfit_field_on_blur(
    mouse: Res<ButtonInput<MouseButton>>,
    field_hover: Query<&Interaction, With<OutfitFieldButton>>,
    mut drafts: ResMut<OutfitFieldDrafts>,
    mut outfits: ResMut<OutfitDbState>,
    mut loaded: ResMut<LoadedImage>,
) {
    let Some(active) = drafts.active else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let hovered_field = field_hover.iter().any(|interaction| {
        *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
    });
    if hovered_field {
        return;
    }

    commit_outfit_field(active, &mut drafts, &mut outfits, &mut loaded);
    drafts.active = None;
}

pub(super) fn handle_outfit_filter_field_focus(
    mut fields: Query<&Interaction, (Changed<Interaction>, With<OutfitFilterFieldButton>)>,
    mut filter: ResMut<OutfitListFilterState>,
) {
    for interaction in &mut fields {
        if *interaction == Interaction::Pressed {
            filter.field_active = true;
        }
    }
}

pub(super) fn handle_outfit_filter_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut filter: ResMut<OutfitListFilterState>,
) {
    if !filter.field_active {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(ch) => {
                let filtered: String = ch
                    .chars()
                    .filter(|c| {
                        c.is_ascii_alphanumeric() || matches!(*c, '_' | '-' | ':' | '/' | ' ')
                    })
                    .map(|c| c.to_ascii_lowercase())
                    .collect();
                if filtered.is_empty() {
                    continue;
                }
                filter.query.push_str(&filtered);
            }
            Key::Backspace => {
                filter.query.pop();
            }
            Key::Enter => {
                add_filter_tag_from_query(&mut filter);
                filter.field_active = false;
            }
            _ => {}
        }
    }
}

pub(super) fn commit_outfit_filter_on_blur(
    mouse: Res<ButtonInput<MouseButton>>,
    field_hover: Query<&Interaction, With<OutfitFilterFieldButton>>,
    mut filter: ResMut<OutfitListFilterState>,
) {
    if !filter.field_active || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let hovered = field_hover.iter().any(|interaction| {
        *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
    });
    if hovered {
        return;
    }
    filter.field_active = false;
}

pub(super) fn handle_outfit_filter_buttons(
    mut add_buttons: Query<&Interaction, (Changed<Interaction>, With<AddOutfitFilterTagButton>)>,
    mut clear_buttons: Query<&Interaction, (Changed<Interaction>, With<ClearOutfitFiltersButton>)>,
    mut filter: ResMut<OutfitListFilterState>,
) {
    for interaction in &mut add_buttons {
        if *interaction == Interaction::Pressed {
            add_filter_tag_from_query(&mut filter);
            filter.field_active = false;
        }
    }

    for interaction in &mut clear_buttons {
        if *interaction == Interaction::Pressed {
            filter.query.clear();
            filter.active_tags.clear();
            filter.field_active = false;
        }
    }
}

pub(super) fn handle_outfit_filter_tag_remove_buttons(
    mut remove_buttons: Query<
        (&Interaction, &OutfitRemoveFilterTagButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut filter: ResMut<OutfitListFilterState>,
) {
    for (interaction, remove) in &mut remove_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if remove.tag_index >= filter.active_tags.len() {
            continue;
        }
        filter.active_tags.remove(remove.tag_index);
        break;
    }
}

pub(super) fn handle_outfit_autocomplete_clicks(
    mut buttons: Query<
        (&Interaction, &OutfitAutocompleteSuggestionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut filter: ResMut<OutfitListFilterState>,
) {
    for (interaction, suggestion) in &mut buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if !filter.active_tags.iter().any(|tag| tag == &suggestion.tag) {
            filter.active_tags.push(suggestion.tag.clone());
        }
        filter.query.clear();
        filter.field_active = false;
        break;
    }
}

pub(super) fn sync_outfit_panel_widgets(
    mut commands: Commands,
    outfits: Res<OutfitDbState>,
    drafts: Res<OutfitFieldDrafts>,
    filter: Res<OutfitListFilterState>,
    paper_doll: Res<PaperDollState>,
    palette_panel: Res<PalettePanelState>,
    layer_palettes: Res<LayerPaletteState>,
    mut containers: ParamSet<(
        Single<Entity, With<OutfitListContainer>>,
        Single<Entity, With<OutfitTagChipsContainer>>,
        Single<Entity, With<OutfitFilterChipsContainer>>,
        Single<Entity, With<OutfitFilterAutocompleteContainer>>,
    )>,
    children_query: Query<&Children>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<OutfitIdentityText>>,
        Query<&mut Text, With<OutfitSummaryText>>,
        Query<&mut Text, With<OutfitStatusText>>,
        Query<(&OutfitFieldText, &mut Text)>,
        Query<(&OutfitFieldButton, &mut BorderColor)>,
        Query<&mut Text, With<OutfitFilterFieldText>>,
        Query<&mut BorderColor, With<OutfitFilterFieldButton>>,
    )>,
) {
    if !outfits.is_changed()
        && !drafts.is_changed()
        && !filter.is_changed()
        && !paper_doll.is_changed()
        && !palette_panel.is_changed()
        && !layer_palettes.is_changed()
    {
        return;
    }

    let list_container = *containers.p0();
    let tags_container = *containers.p1();
    let filter_tags_container = *containers.p2();
    let autocomplete_container = *containers.p3();

    for (field_text, mut text) in &mut text_queries.p3() {
        *text = Text::new(drafts.value(field_text.field));
    }
    for (field_button, mut border) in &mut text_queries.p4() {
        let color = if field_button.field == OutfitFieldKind::OutfitId && drafts.invalid_outfit_id {
            Color::srgb(0.82, 0.18, 0.18)
        } else if drafts.active == Some(field_button.field) {
            Color::srgb(0.34, 0.56, 0.98)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.22)
        };
        *border = BorderColor::all(color);
    }
    for mut text in &mut text_queries.p5() {
        *text = Text::new(filter.query.clone());
    }
    for mut border in &mut text_queries.p6() {
        let color = if filter.field_active {
            Color::srgb(0.34, 0.56, 0.98)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.22)
        };
        *border = BorderColor::all(color);
    }

    let selected_outfit = outfits
        .selected
        .and_then(|index| outfits.db.outfits.get(index));
    for mut text in &mut text_queries.p0() {
        let label = selected_outfit.map_or_else(
            || "Selected: (none)".to_string(),
            |outfit| format!("Selected: {}", outfit.outfit_id),
        );
        *text = Text::new(label);
    }

    for mut text in &mut text_queries.p2() {
        *text = Text::new(format!(
            "Outfits: {} | Visible: {} | Dirty: {} | File: {}",
            outfits.db.outfits.len(),
            filtered_outfit_count(outfits.as_ref(), filter.as_ref()),
            if outfits.dirty { "yes" } else { "no" },
            outfits.path.display()
        ));
    }

    let summary = build_preview_summary(&paper_doll, &palette_panel, &layer_palettes);
    for mut text in &mut text_queries.p1() {
        *text = Text::new(summary.clone());
    }

    despawn_children_of(&mut commands, list_container, &children_query);
    commands.entity(list_container).with_children(|list| {
        let visible_indices = filtered_outfit_indices(outfits.as_ref(), filter.as_ref());
        if visible_indices.is_empty() {
            list.spawn((Text::new("(no outfits)"), TextFont::from_font_size(10.0)));
            return;
        }
        for index in visible_indices {
            let Some(outfit) = outfits.db.outfits.get(index) else {
                continue;
            };
            let selected = outfits.selected == Some(index);
            let label = if outfit.display_name.trim().is_empty() {
                outfit.outfit_id.clone()
            } else {
                outfit.display_name.clone()
            };
            let tag_line = if outfit.tags.is_empty() {
                String::new()
            } else {
                outfit
                    .tags
                    .iter()
                    .map(|tag| format!("[{tag}]"))
                    .collect::<Vec<String>>()
                    .join(" ")
            };
            list.spawn((
                Button,
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    padding: UiRect::all(px(6)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(if selected {
                    Color::srgb(0.20, 0.20, 0.14)
                } else {
                    Color::srgb(0.12, 0.12, 0.15)
                }),
                BorderColor::all(if selected {
                    Color::srgb(0.95, 0.82, 0.22)
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.20)
                }),
                OutfitListItemButton { index },
            ))
            .with_children(|item| {
                item.spawn((Text::new(label),));
                if !tag_line.is_empty() {
                    item.spawn((Text::new(tag_line), TextFont::from_font_size(10.0)));
                }
            });
        }
    });

    despawn_children_of(&mut commands, filter_tags_container, &children_query);
    commands
        .entity(filter_tags_container)
        .with_children(|chips| {
            if filter.active_tags.is_empty() {
                chips.spawn((
                    Text::new("(no filter tags)"),
                    TextFont::from_font_size(10.0),
                ));
                return;
            }
            for (tag_index, tag) in filter.active_tags.iter().enumerate() {
                chips
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(px(6), px(4)),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.18, 0.16)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                        OutfitRemoveFilterTagButton { tag_index },
                    ))
                    .with_children(|chip| {
                        chip.spawn((
                            Text::new(format!("{tag}  x")),
                            TextFont::from_font_size(10.0),
                        ));
                    });
            }
        });

    despawn_children_of(&mut commands, autocomplete_container, &children_query);
    commands
        .entity(autocomplete_container)
        .with_children(|suggestions| {
            let tags = autocomplete_tags(outfits.as_ref(), filter.as_ref());
            if tags.is_empty() {
                suggestions.spawn((
                    Text::new("(no suggestions)"),
                    TextFont::from_font_size(10.0),
                ));
                return;
            }
            for tag in tags {
                suggestions
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(px(6), px(4)),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.14, 0.14, 0.18)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                        OutfitAutocompleteSuggestionButton { tag: tag.clone() },
                    ))
                    .with_children(|chip| {
                        chip.spawn((
                            Text::new(format!("+ {tag}")),
                            TextFont::from_font_size(10.0),
                        ));
                    });
            }
        });

    despawn_children_of(&mut commands, tags_container, &children_query);
    commands.entity(tags_container).with_children(|chips| {
        let Some(outfit) = selected_outfit else {
            chips.spawn((Text::new("(no tags)"), TextFont::from_font_size(10.0)));
            return;
        };
        if outfit.tags.is_empty() {
            chips.spawn((Text::new("(no tags)"), TextFont::from_font_size(10.0)));
            return;
        }
        for (tag_index, tag) in outfit.tags.iter().enumerate() {
            chips
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(px(6), px(4)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.14, 0.14, 0.18)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                    OutfitRemoveTagButton { tag_index },
                ))
                .with_children(|chip| {
                    chip.spawn((
                        Text::new(format!("{tag}  x")),
                        TextFont::from_font_size(10.0),
                    ));
                });
        }
    });
}

pub(super) fn commit_outfit_field(
    field: OutfitFieldKind,
    drafts: &mut OutfitFieldDrafts,
    outfits: &mut OutfitDbState,
    loaded: &mut LoadedImage,
) {
    let Some(selected) = outfits.selected else {
        loaded.status = Some("Outfits: select an outfit first".to_string());
        return;
    };
    let Some(outfit) = outfits.db.outfits.get(selected) else {
        return;
    };

    match field {
        OutfitFieldKind::OutfitId => {
            let next_id = sanitize_outfit_id(&drafts.outfit_id);
            if next_id.is_empty() {
                drafts.invalid_outfit_id = true;
                loaded.status =
                    Some("Outfit ID must contain letters/digits/underscores".to_string());
                return;
            }
            if outfits
                .db
                .outfits
                .iter()
                .enumerate()
                .any(|(index, candidate)| index != selected && candidate.outfit_id == next_id)
            {
                drafts.invalid_outfit_id = true;
                loaded.status = Some("Outfit ID must be unique".to_string());
                return;
            }
            drafts.invalid_outfit_id = false;
            if let Some(slot) = outfits.db.outfits.get_mut(selected) {
                slot.outfit_id = next_id.clone();
            }
            drafts.outfit_id = next_id.clone();
            outfits.dirty = true;
            loaded.status = Some(format!("Outfits: updated ID to {next_id}"));
        }
        OutfitFieldKind::DisplayName => {
            let next_display = if drafts.display_name.trim().is_empty() {
                outfit.outfit_id.clone()
            } else {
                drafts.display_name.trim().to_string()
            };
            if let Some(slot) = outfits.db.outfits.get_mut(selected) {
                slot.display_name = next_display.clone();
            }
            drafts.display_name = next_display.clone();
            outfits.dirty = true;
            loaded.status = Some(format!("Outfits: updated display name to {next_display}"));
        }
        OutfitFieldKind::TagInput => {
            let tag = normalize_tag(&drafts.tag_input);
            if tag.is_empty() {
                return;
            }
            if let Some(slot) = outfits.db.outfits.get_mut(selected) {
                if !slot.tags.iter().any(|existing| existing == &tag) {
                    slot.tags.push(tag.clone());
                    outfits.dirty = true;
                    loaded.status = Some(format!("Outfits: added tag '{tag}'"));
                }
            }
            drafts.tag_input.clear();
        }
    }
}

fn filter_outfit_input(field: OutfitFieldKind, input: &str) -> String {
    match field {
        OutfitFieldKind::OutfitId => input
            .chars()
            .filter_map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    Some(ch.to_ascii_lowercase())
                } else if matches!(ch, '_' | '-' | ' ') {
                    Some('_')
                } else {
                    None
                }
            })
            .collect(),
        OutfitFieldKind::DisplayName => input
            .chars()
            .filter(|ch| ch.is_ascii_graphic() || *ch == ' ')
            .collect(),
        OutfitFieldKind::TagInput => input
            .chars()
            .filter_map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    Some(ch.to_ascii_lowercase())
                } else if matches!(ch, '_' | '-' | ':' | '/') {
                    Some(ch)
                } else {
                    None
                }
            })
            .collect(),
    }
}

fn add_filter_tag_from_query(filter: &mut OutfitListFilterState) {
    let tag = normalize_tag(&filter.query);
    if tag.is_empty() {
        return;
    }
    if !filter.active_tags.iter().any(|existing| existing == &tag) {
        filter.active_tags.push(tag);
    }
    filter.query.clear();
}

fn filtered_outfit_count(outfits: &OutfitDbState, filter: &OutfitListFilterState) -> usize {
    filtered_outfit_indices(outfits, filter).len()
}

fn filtered_outfit_indices(outfits: &OutfitDbState, filter: &OutfitListFilterState) -> Vec<usize> {
    outfits
        .db
        .outfits
        .iter()
        .enumerate()
        .filter_map(|(index, outfit)| outfit_matches_filter(outfit, filter).then_some(index))
        .collect()
}

fn outfit_matches_filter(outfit: &Outfit, filter: &OutfitListFilterState) -> bool {
    let query = filter.query.trim().to_ascii_lowercase();
    let query_match = query.is_empty()
        || outfit.outfit_id.to_ascii_lowercase().contains(&query)
        || outfit.display_name.to_ascii_lowercase().contains(&query)
        || outfit
            .tags
            .iter()
            .any(|tag| tag.to_ascii_lowercase().contains(&query));

    if !query_match {
        return false;
    }

    filter.active_tags.iter().all(|required| {
        let required = normalize_tag(required);
        if required.is_empty() {
            return true;
        }
        outfit.tags.iter().any(|tag| normalize_tag(tag) == required)
    })
}

fn autocomplete_tags(outfits: &OutfitDbState, filter: &OutfitListFilterState) -> Vec<String> {
    let query = normalize_tag(&filter.query);
    let mut unique = std::collections::BTreeSet::new();
    for outfit in &outfits.db.outfits {
        for tag in &outfit.tags {
            let normalized = normalize_tag(tag);
            if normalized.is_empty() {
                continue;
            }
            if filter
                .active_tags
                .iter()
                .any(|active| active == &normalized)
            {
                continue;
            }
            if !query.is_empty() && !normalized.contains(&query) {
                continue;
            }
            unique.insert(normalized);
        }
    }
    unique.into_iter().take(12).collect()
}

fn snapshot_outfit_from_preview(
    paper_doll: &PaperDollState,
    palette_panel: &PalettePanelState,
    layer_palettes: &LayerPaletteState,
    outfit_id: String,
    display_name: String,
    tags: Vec<String>,
) -> Outfit {
    let global = global_palette_choice(palette_panel);
    let skin = choice_for_layer(LayerCode::Body01, layer_palettes, global);
    let hair = choice_for_layer(LayerCode::Hair13, layer_palettes, global);
    let outfit_main = global;
    let outfit_accent = layer_palettes.by_layer.get(&LayerCode::Outr10).copied();

    let equipped = LayerCode::ALL
        .into_iter()
        .filter_map(|layer| {
            let index = paper_doll.equipped.by_layer.get(&layer)?;
            let part = paper_doll.catalog.parts.get(*index)?;
            Some(OutfitEquippedPart {
                layer,
                part_key: part.part_key.clone(),
            })
        })
        .collect();

    Outfit {
        outfit_id,
        display_name,
        tags,
        equipped,
        palette: OutfitPaletteSelection {
            skin: RampChoice::Preset(encode_ramp_preset(skin)),
            hair: RampChoice::Preset(encode_ramp_preset(hair)),
            outfit_main: RampChoice::Preset(encode_ramp_preset(outfit_main)),
            outfit_accent: outfit_accent
                .map(|selection| RampChoice::Preset(encode_ramp_preset(selection))),
        },
    }
}

fn apply_outfit_to_preview(
    outfit: &Outfit,
    paper_doll: &mut PaperDollState,
    palette_panel: &mut PalettePanelState,
    layer_palettes: &mut LayerPaletteState,
) {
    let main = parse_ramp_choice(&outfit.palette.outfit_main)
        .unwrap_or_else(|| global_palette_choice(palette_panel));
    palette_panel.selected = Some(main.palette_index);
    palette_panel.variant = main.variant;

    layer_palettes.by_layer.clear();
    if let Some(skin) = parse_ramp_choice(&outfit.palette.skin) {
        if skin != main {
            layer_palettes.by_layer.insert(LayerCode::Body01, skin);
        }
    }
    if let Some(hair) = parse_ramp_choice(&outfit.palette.hair) {
        if hair != main {
            layer_palettes.by_layer.insert(LayerCode::Hair13, hair);
        }
    }
    if let Some(accent_choice) = outfit
        .palette
        .outfit_accent
        .as_ref()
        .and_then(parse_ramp_choice)
    {
        if accent_choice != main {
            layer_palettes
                .by_layer
                .insert(LayerCode::Outr10, accent_choice);
        }
    }

    let keys: Vec<String> = outfit
        .equipped
        .iter()
        .map(|entry| entry.part_key.clone())
        .collect();
    if paper_doll.loaded {
        let PaperDollState {
            catalog, equipped, ..
        } = paper_doll;
        equipped.apply_equipped_keys(catalog, &keys);
    } else {
        paper_doll.pending_equipped_keys = Some(keys);
    }
}

fn next_outfit_id(outfits: &OutfitDbState) -> String {
    let mut counter = 1usize;
    loop {
        let candidate = format!("new_outfit_{counter:03}");
        if !outfits
            .db
            .outfits
            .iter()
            .any(|outfit| outfit.outfit_id == candidate)
        {
            return candidate;
        }
        counter += 1;
    }
}

fn sanitize_outfit_id(raw: &str) -> String {
    let mut id = String::new();
    let mut last_was_sep = false;
    for ch in raw.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else if matches!(ch, '_' | '-' | ' ') {
            Some('_')
        } else {
            None
        };
        let Some(mapped) = mapped else {
            continue;
        };
        if mapped == '_' {
            if id.is_empty() || last_was_sep {
                continue;
            }
            last_was_sep = true;
            id.push(mapped);
        } else {
            last_was_sep = false;
            id.push(mapped);
        }
    }
    id.trim_matches('_').to_string()
}

fn normalize_tag(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if matches!(ch, '_' | '-' | ':' | '/') {
                Some(ch)
            } else {
                None
            }
        })
        .collect()
}

fn global_palette_choice(panel: &PalettePanelState) -> LayerPaletteSelection {
    LayerPaletteSelection {
        palette_index: panel.selected.unwrap_or(0),
        variant: panel.variant,
    }
}

fn choice_for_layer(
    layer: LayerCode,
    layer_palettes: &LayerPaletteState,
    global: LayerPaletteSelection,
) -> LayerPaletteSelection {
    layer_palettes
        .by_layer
        .get(&layer)
        .copied()
        .unwrap_or(global)
}

fn encode_ramp_preset(selection: LayerPaletteSelection) -> String {
    format!(
        "palette:{}:variant:{}",
        selection.palette_index, selection.variant
    )
}

fn parse_ramp_choice(choice: &RampChoice) -> Option<LayerPaletteSelection> {
    match choice {
        RampChoice::Preset(raw) => parse_ramp_preset(raw),
        RampChoice::Custom(_) => None,
    }
}

fn parse_ramp_preset(raw: &str) -> Option<LayerPaletteSelection> {
    let mut iter = raw.split(':');
    let key = iter.next()?;
    if key != "palette" {
        return None;
    }
    let palette_index = iter.next()?.parse::<usize>().ok()?;
    let variant_key = iter.next()?;
    if variant_key != "variant" {
        return None;
    }
    let variant = iter.next()?.parse::<usize>().ok()?;
    Some(LayerPaletteSelection {
        palette_index,
        variant,
    })
}

fn build_preview_summary(
    paper_doll: &PaperDollState,
    palette_panel: &PalettePanelState,
    layer_palettes: &LayerPaletteState,
) -> String {
    let mut lines = Vec::new();
    lines.push("Equipped parts (preview):".to_string());
    for layer in LayerCode::ALL {
        let Some(index) = paper_doll.equipped.by_layer.get(&layer) else {
            continue;
        };
        let Some(part) = paper_doll.catalog.parts.get(*index) else {
            continue;
        };
        lines.push(format!("{}: {}", layer.as_str(), part.part_key));
    }
    if lines.len() == 1 {
        lines.push("(none)".to_string());
    }

    let global = global_palette_choice(palette_panel);
    let skin = choice_for_layer(LayerCode::Body01, layer_palettes, global);
    let hair = choice_for_layer(LayerCode::Hair13, layer_palettes, global);
    let accent = layer_palettes.by_layer.get(&LayerCode::Outr10).copied();
    lines.push(String::new());
    lines.push("Palette snapshot (preview):".to_string());
    lines.push(format!("skin: {}", encode_ramp_preset(skin)));
    lines.push(format!("hair: {}", encode_ramp_preset(hair)));
    lines.push(format!("outfit_main: {}", encode_ramp_preset(global)));
    lines.push(format!(
        "outfit_accent: {}",
        accent
            .map(encode_ramp_preset)
            .unwrap_or_else(|| "(none)".to_string())
    ));
    lines.join("\n")
}

fn despawn_children_of(commands: &mut Commands, parent: Entity, children_query: &Query<&Children>) {
    let Ok(children) = children_query.get(parent) else {
        return;
    };
    for child in children.iter() {
        commands.entity(child).despawn();
    }
}

pub(super) fn handle_grid_field_focus(
    mut fields: Query<(&Interaction, &GridFieldButton), (Changed<Interaction>, With<Button>)>,
    mut drafts: ResMut<GridFieldDrafts>,
    mut grid: ResMut<GridState>,
    mut selection_state: ResMut<SelectionState>,
    mut loaded: ResMut<LoadedImage>,
    mut canvas_ui: ResMut<CanvasUiState>,
) {
    for (interaction, field) in &mut fields {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(active) = drafts.active
            && active != field.field
        {
            commit_grid_field(
                active,
                &mut drafts,
                &mut grid,
                &mut selection_state,
                &mut loaded,
                &mut canvas_ui,
            );
        }
        drafts.active = Some(field.field);
    }
}

pub(super) fn handle_grid_field_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut drafts: ResMut<GridFieldDrafts>,
    mut grid: ResMut<GridState>,
    mut selection_state: ResMut<SelectionState>,
    mut loaded: ResMut<LoadedImage>,
    mut canvas_ui: ResMut<CanvasUiState>,
) {
    let Some(active) = drafts.active else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(ch) => {
                let digits: String = ch.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.is_empty() {
                    continue;
                }
                drafts.value_mut(active).push_str(&digits);
                drafts.set_invalid(active, false);
            }
            Key::Backspace => {
                drafts.value_mut(active).pop();
                drafts.set_invalid(active, false);
            }
            Key::Enter => {
                commit_grid_field(
                    active,
                    &mut drafts,
                    &mut grid,
                    &mut selection_state,
                    &mut loaded,
                    &mut canvas_ui,
                );
                drafts.active = None;
            }
            _ => {}
        }
    }
}

#[derive(SystemParam)]
pub(super) struct GridFieldBlurCtx<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    input_context: Res<'w, InputContext>,
    field_hover: Query<'w, 's, &'static Interaction, With<GridFieldButton>>,
    drafts: ResMut<'w, GridFieldDrafts>,
    grid: ResMut<'w, GridState>,
    selection_state: ResMut<'w, SelectionState>,
    loaded: ResMut<'w, LoadedImage>,
    canvas_ui: ResMut<'w, CanvasUiState>,
}

pub(super) fn commit_grid_field_on_blur(ctx: GridFieldBlurCtx) {
    let GridFieldBlurCtx {
        mouse,
        input_context,
        field_hover,
        mut drafts,
        mut grid,
        mut selection_state,
        mut loaded,
        mut canvas_ui,
    } = ctx;

    let Some(active) = drafts.active else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let hovered_field = field_hover.iter().any(|interaction| {
        *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
    });
    if hovered_field {
        return;
    }

    commit_grid_field(
        active,
        &mut drafts,
        &mut grid,
        &mut selection_state,
        &mut loaded,
        &mut canvas_ui,
    );
    drafts.active = None;

    if input_context.hovered_region != Some(InputRegion::LeftPanel) {
        loaded.status = Some("Grid settings applied".to_string());
    }
}

pub(super) fn sync_grid_fields_from_state(
    grid: Res<GridState>,
    mut drafts: ResMut<GridFieldDrafts>,
) {
    if !grid.is_changed() {
        return;
    }
    let keep_active = drafts.active;
    if keep_active != Some(GridFieldKind::Rows) {
        drafts.rows = grid.rows.to_string();
    }
    if keep_active != Some(GridFieldKind::Columns) {
        drafts.columns = grid.columns.to_string();
    }
    if keep_active != Some(GridFieldKind::CellWidth) {
        drafts.cell_width = grid.cell_width.to_string();
    }
    if keep_active != Some(GridFieldKind::CellHeight) {
        drafts.cell_height = grid.cell_height.to_string();
    }
    if keep_active != Some(GridFieldKind::OffsetX) {
        drafts.offset_x = grid.offset_x.to_string();
    }
    if keep_active != Some(GridFieldKind::OffsetY) {
        drafts.offset_y = grid.offset_y.to_string();
    }
}

pub(super) fn section_is_open(
    sections: &LeftPanelSectionsState,
    section: LeftPanelSection,
) -> bool {
    match section {
        LeftPanelSection::SpriteSheet => sections.sprite_sheet_open,
        LeftPanelSection::GridSettings => sections.grid_settings_open,
        LeftPanelSection::Parts => sections.parts_open,
        LeftPanelSection::Palettes => sections.palettes_open,
        LeftPanelSection::Animations => sections.animations_open,
    }
}

pub(super) fn set_section_open(
    sections: &mut LeftPanelSectionsState,
    section: LeftPanelSection,
    open: bool,
) {
    match section {
        LeftPanelSection::SpriteSheet => sections.sprite_sheet_open = open,
        LeftPanelSection::GridSettings => sections.grid_settings_open = open,
        LeftPanelSection::Parts => sections.parts_open = open,
        LeftPanelSection::Palettes => sections.palettes_open = open,
        LeftPanelSection::Animations => sections.animations_open = open,
    }
}

pub(super) fn commit_grid_field(
    field: GridFieldKind,
    drafts: &mut GridFieldDrafts,
    grid: &mut GridState,
    selection_state: &mut SelectionState,
    loaded: &mut LoadedImage,
    canvas_ui: &mut CanvasUiState,
) {
    let raw = drafts.value(field).trim();
    let Ok(parsed) = raw.parse::<u32>() else {
        drafts.set_invalid(field, true);
        loaded.status = Some("Grid field must be numeric".to_string());
        return;
    };

    match field {
        GridFieldKind::Rows => grid.rows = parsed,
        GridFieldKind::Columns => grid.columns = parsed,
        GridFieldKind::CellWidth => grid.cell_width = parsed,
        GridFieldKind::CellHeight => grid.cell_height = parsed,
        GridFieldKind::OffsetX => grid.offset_x = parsed,
        GridFieldKind::OffsetY => grid.offset_y = parsed,
    }
    grid.normalize();
    drafts.set_invalid(field, false);

    let cell_limit = grid_state::cell_count(grid);
    selection_state
        .selected_cells
        .retain(|index| *index < cell_limit);
    canvas_ui.dirty = true;
    loaded.status = Some("Grid settings applied".to_string());
}
