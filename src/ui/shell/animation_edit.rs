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
