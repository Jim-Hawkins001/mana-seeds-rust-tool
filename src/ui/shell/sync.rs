use super::*;
use bevy::ecs::system::SystemParam;

type ViewerDisplayTexts<'w, 's> = ParamSet<
    'w,
    's,
    (
        Query<'w, 's, &'static mut Text, With<ViewerClipText>>,
        Query<'w, 's, &'static mut Text, With<ViewerCellText>>,
        Query<'w, 's, &'static mut Text, With<ViewerStepText>>,
        Query<'w, 's, &'static mut Text, With<ViewerMsText>>,
        Query<'w, 's, &'static mut Text, With<ViewerFlipText>>,
    ),
>;

type ViewerImageNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static ViewerLayerImageNode,
        &'static mut ImageNode,
        &'static mut Node,
    ),
>;

#[derive(SystemParam)]
pub(super) struct RebuildCanvasCtx<'w, 's> {
    commands: Commands<'w, 's>,
    canvas_ui: ResMut<'w, CanvasUiState>,
    canvas_surface: Single<'w, 's, Entity, With<CanvasSurface>>,
    viewport: Single<'w, 's, &'static ComputedNode, With<CanvasViewport>>,
    grid: Res<'w, GridState>,
    loaded: Res<'w, LoadedImage>,
    canvas_view: ResMut<'w, CanvasView>,
    selection_state: Res<'w, SelectionState>,
}

#[derive(SystemParam)]
pub(super) struct ViewerDisplayCtx<'w, 's> {
    anim_state: Res<'w, AnimationAuthoringState>,
    viewer: Res<'w, AnimViewerState>,
    preview_atlas: Res<'w, PreviewAtlasState>,
    paper_doll: Res<'w, PaperDollState>,
    palette_panel: Res<'w, PalettePanelState>,
    layer_palettes: Res<'w, LayerPaletteState>,
    palette_remap_cache: ResMut<'w, PaletteRemapCache>,
    images: ResMut<'w, Assets<Image>>,
    atlas_layouts: Res<'w, Assets<TextureAtlasLayout>>,
    viewer_images: ViewerImageNodes<'w, 's>,
    text_sets: ViewerDisplayTexts<'w, 's>,
}

pub(super) fn mark_canvas_dirty_on_data_change(
    grid: Res<GridState>,
    loaded: Res<LoadedImage>,
    mut canvas_ui: ResMut<CanvasUiState>,
) {
    if grid.is_changed() || loaded.is_changed() {
        canvas_ui.dirty = true;
    }
}

pub(super) fn rebuild_canvas_if_needed(ctx: RebuildCanvasCtx) {
    let RebuildCanvasCtx {
        mut commands,
        mut canvas_ui,
        canvas_surface,
        viewport,
        grid,
        loaded,
        mut canvas_view,
        selection_state,
    } = ctx;

    if !canvas_ui.dirty {
        return;
    }

    for entity in canvas_ui.dynamic_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    let total_size = canvas_content_size(&grid, &loaded);
    let scaled_size = total_size * canvas_view.zoom;
    let viewport_logical_size = viewport.size() * viewport.inverse_scale_factor();
    canvas_view.offset =
        clamp_canvas_offset(canvas_view.offset, scaled_size, viewport_logical_size);

    commands.entity(*canvas_surface).insert(Node {
        width: px(scaled_size.x),
        height: px(scaled_size.y),
        position_type: PositionType::Absolute,
        left: px(canvas_view.offset.x),
        top: px(canvas_view.offset.y),
        ..default()
    });

    let mut created = Vec::new();
    commands.entity(*canvas_surface).with_children(|surface| {
        let image_size = loaded
            .size
            .map(|size| Vec2::new(size.x as f32, size.y as f32))
            .unwrap_or(Vec2::ZERO);
        if let Some(handle) = loaded.handle.as_ref() {
            let image_entity = surface
                .spawn((
                    ImageNode::new(handle.clone()),
                    Node {
                        width: px((image_size.x * canvas_view.zoom).max(1.0)),
                        height: px((image_size.y * canvas_view.zoom).max(1.0)),
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        ..default()
                    },
                ))
                .id();
            created.push(image_entity);
        }

        for row in 0..grid.rows {
            for column in 0..grid.columns {
                let index = grid_state::cell_index(row, column, grid.columns);
                let selected = selection_state.selected_cells.contains(&index);
                let left = grid.offset_x as f32 + column as f32 * grid.cell_width as f32;
                let top = grid.offset_y as f32 + row as f32 * grid.cell_height as f32;
                let color = if selected {
                    overlay::selected_fill_color()
                } else {
                    Color::NONE
                };
                let border = if selected {
                    overlay::selected_border_color()
                } else {
                    overlay::grid_line_color()
                };

                let cell_entity = surface
                    .spawn((
                        Node {
                            width: px(grid.cell_width as f32 * canvas_view.zoom),
                            height: px(grid.cell_height as f32 * canvas_view.zoom),
                            position_type: PositionType::Absolute,
                            left: px(left * canvas_view.zoom),
                            top: px(top * canvas_view.zoom),
                            border: UiRect::all(px(1)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(color),
                        BorderColor::all(border),
                        overlay::GridOverlayMarker,
                        GridCellButton { index },
                    ))
                    .with_children(|button| {
                        button.spawn((Text::new(index.to_string()),));
                    })
                    .id();
                created.push(cell_entity);
            }
        }
    });

    canvas_ui.dynamic_entities = created;
    canvas_ui.dirty = false;
}

pub(super) fn sync_mode_text(
    mode: Res<EditorMode>,
    mut labels: Query<&mut Text, With<ModeTextLabel>>,
) {
    if !mode.is_changed() {
        return;
    }
    for mut text in &mut labels {
        *text = Text::new(top_toolbar::mode_text(*mode));
    }
}

pub(super) fn sync_selection_text(
    selection_state: Res<SelectionState>,
    mut labels: Query<&mut Text, With<SelectionCountText>>,
) {
    if !selection_state.is_changed() {
        return;
    }
    for mut text in &mut labels {
        *text = Text::new(format!(
            "Selected Cells: {}",
            selection_state.selected_cells.len()
        ));
    }
}

pub(super) fn sync_grid_field_widgets(
    drafts: Res<GridFieldDrafts>,
    mut field_texts: Query<(&GridFieldText, &mut Text)>,
    mut field_borders: Query<(&GridFieldButton, &mut BorderColor)>,
) {
    if !drafts.is_changed() {
        return;
    }

    for (field_text, mut text) in &mut field_texts {
        *text = Text::new(drafts.value(field_text.field));
    }

    for (field_button, mut border) in &mut field_borders {
        let color = if drafts.is_invalid(field_button.field) {
            Color::srgb(0.82, 0.18, 0.18)
        } else if drafts.active == Some(field_button.field) {
            Color::srgb(0.34, 0.56, 0.98)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.22)
        };
        *border = BorderColor::all(color);
    }
}

pub(super) fn sync_left_panel_texts(
    grid: Res<GridState>,
    loaded: Res<LoadedImage>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<ImageNameText>>,
        Query<&mut Text, With<ImageSizeText>>,
        Query<&mut Text, With<StatusText>>,
    )>,
) {
    if !loaded.is_changed() && !grid.is_changed() {
        return;
    }

    if let Some(mut text) = text_queries.p0().iter_mut().next() {
        *text = Text::new(format!(
            "Image: {}",
            loaded.name.as_deref().unwrap_or("(none)")
        ));
    }
    if let Some(mut text) = text_queries.p1().iter_mut().next() {
        let value = loaded
            .size
            .map(|size| format!("{}x{}", size.x, size.y))
            .unwrap_or_else(|| "(none)".to_string());
        *text = Text::new(format!("Resolution: {value}"));
    }
    if let Some(mut text) = text_queries.p2().iter_mut().next() {
        *text = Text::new(format!(
            "Status: {}",
            loaded.status.as_deref().unwrap_or("idle")
        ));
    }
}

pub(super) fn sync_animation_tree_visibility(
    panel_state: Res<AnimationPanelState>,
    anim_state: Res<AnimationAuthoringState>,
    mut category_bodies: Query<(&AnimationCategoryBody, &mut Node)>,
    mut category_arrows: Query<(&AnimationCategoryArrowText, &mut Text)>,
    mut clip_buttons: Query<(&AnimationClipButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    if !panel_state.is_changed() && !anim_state.is_changed() {
        return;
    }

    for (body, mut node) in &mut category_bodies {
        node.display = if panel_state.expanded_categories.contains(&body.category) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (arrow, mut text) in &mut category_arrows {
        let marker = if panel_state.expanded_categories.contains(&arrow.category) {
            "v"
        } else {
            ">"
        };
        *text = Text::new(marker);
    }

    for (clip_button, mut border, mut bg) in &mut clip_buttons {
        let selected = clip_button.clip_index == anim_state.active_clip;
        *border = BorderColor::all(if selected {
            Color::srgb(0.88, 0.78, 0.20)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.18)
        });
        *bg = BackgroundColor(if selected {
            Color::srgb(0.20, 0.20, 0.14)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
    }
}

pub(super) fn sync_direction_button_styles(
    panel_state: Res<AnimationPanelState>,
    mut direction_buttons: Query<(
        &DirectionButtonStyleMarker,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
) {
    if !panel_state.is_changed() {
        return;
    }
    for (marker, mut border, mut bg) in &mut direction_buttons {
        let is_left = marker.direction == Direction::Left;
        let is_active = panel_state.active_dir == Some(marker.direction);
        *bg = BackgroundColor(if is_active {
            Color::srgb(0.34, 0.30, 0.12)
        } else if is_left {
            Color::srgb(0.10, 0.10, 0.12)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
        *border = BorderColor::all(if is_active {
            Color::srgb(0.95, 0.82, 0.22)
        } else if is_left {
            Color::srgba(1.0, 1.0, 1.0, 0.10)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.20)
        });
    }
}

pub(super) fn sync_anim_viewer_clip_binding(
    anim_state: Res<AnimationAuthoringState>,
    mut viewer: ResMut<AnimViewerState>,
) {
    if !anim_state.is_changed() {
        return;
    }
    let Some(clip) = anim_state.active_clip() else {
        viewer.clip_id = None;
        reset_viewer_playback(&mut viewer);
        return;
    };
    if viewer.clip_id.as_deref() != Some(clip.id.as_str()) {
        viewer.clip_id = Some(clip.id.clone());
        reset_viewer_playback(&mut viewer);
    }
}

pub(super) fn tick_anim_viewer_playback(
    time: Res<Time>,
    anim_state: Res<AnimationAuthoringState>,
    mut viewer: ResMut<AnimViewerState>,
) {
    let Some(clip) = anim_state.active_clip() else {
        return;
    };
    let Some(resolved) = resolve_viewer_track(clip, viewer.dir) else {
        viewer.step_index = 0;
        viewer.step_elapsed_ms = 0.0;
        viewer.loop_n_remaining = None;
        return;
    };
    let steps = &resolved.track.steps;
    if steps.is_empty() {
        viewer.step_index = 0;
        viewer.step_elapsed_ms = 0.0;
        viewer.loop_n_remaining = None;
        return;
    }

    if viewer.step_index >= steps.len() {
        viewer.step_index = steps.len() - 1;
        viewer.step_elapsed_ms = 0.0;
    }

    if !matches!(clip.playback, Playback::LoopN { .. }) {
        viewer.loop_n_remaining = None;
    }

    if !viewer.is_playing {
        return;
    }

    viewer.step_elapsed_ms += time.delta_secs() * 1000.0 * viewer.speed.max(0.1);

    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let step_ms = steps[viewer.step_index].ms.max(1) as f32;
        if viewer.step_elapsed_ms < step_ms {
            break;
        }
        viewer.step_elapsed_ms -= step_ms;

        if viewer.step_index + 1 < steps.len() {
            viewer.step_index += 1;
            continue;
        }

        if viewer.loop_override {
            viewer.step_index = 0;
            viewer.loop_n_remaining = None;
            continue;
        }

        match clip.playback {
            Playback::Loop => {
                viewer.step_index = 0;
            }
            Playback::LoopN { times } => {
                let remaining = viewer
                    .loop_n_remaining
                    .get_or_insert(times.saturating_sub(1));
                if *remaining > 0 {
                    *remaining -= 1;
                    viewer.step_index = 0;
                } else {
                    viewer.step_index = steps.len() - 1;
                    viewer.step_elapsed_ms = 0.0;
                    viewer.is_playing = false;
                    break;
                }
            }
            Playback::OneShot { .. } => {
                viewer.step_index = steps.len() - 1;
                viewer.step_elapsed_ms = 0.0;
                viewer.is_playing = false;
                break;
            }
        }
    }
}

pub(super) fn sync_anim_viewer_display(ctx: ViewerDisplayCtx) {
    let ViewerDisplayCtx {
        anim_state,
        viewer,
        preview_atlas,
        paper_doll,
        palette_panel,
        layer_palettes,
        mut palette_remap_cache,
        mut images,
        atlas_layouts,
        mut viewer_images,
        mut text_sets,
    } = ctx;

    if !anim_state.is_changed()
        && !viewer.is_changed()
        && !preview_atlas.is_changed()
        && !paper_doll.is_changed()
        && !palette_panel.is_changed()
        && !layer_palettes.is_changed()
        && !atlas_layouts.is_changed()
    {
        return;
    }

    let mut clip_line = "Clip: (none)".to_string();
    let mut cell_line = "Cell: -".to_string();
    let mut step_line = "Step: 0/0".to_string();
    let mut ms_line = "Ms: -".to_string();
    let mut flip_line = "Flip: -".to_string();
    let mut atlas_render: Option<(usize, bool)> = None;

    if let Some(clip) = anim_state.active_clip() {
        clip_line = format!("Clip: {} ({})", clip.display, viewer.dir.label());
        if let Some(resolved) = resolve_viewer_track(clip, viewer.dir) {
            let len = resolved.track.steps.len();
            if len > 0 {
                let index = viewer.step_index.min(len - 1);
                let step = resolved.track.steps[index];
                let effective_flip = step.flip_x ^ resolved.force_flip_x;
                cell_line = format!("Cell: {}", step.cell);
                step_line = format!("Step: {}/{}", index + 1, len);
                ms_line = format!("Ms: {}", step.ms);
                flip_line = if effective_flip {
                    "Flip: X".to_string()
                } else {
                    "Flip: -".to_string()
                };
                atlas_render = Some((step.cell as usize, effective_flip));
            } else {
                step_line = "Step: 0/0".to_string();
            }
        } else {
            step_line = "Step: no track".to_string();
        }
    }

    for mut text in &mut text_sets.p0() {
        *text = Text::new(clip_line.clone());
    }
    for mut text in &mut text_sets.p1() {
        *text = Text::new(cell_line.clone());
    }
    for mut text in &mut text_sets.p2() {
        *text = Text::new(step_line.clone());
    }
    for mut text in &mut text_sets.p3() {
        *text = Text::new(ms_line.clone());
    }
    for mut text in &mut text_sets.p4() {
        *text = Text::new(flip_line.clone());
    }

    let body_atlas_data = match (preview_atlas.image.as_ref(), preview_atlas.layout.as_ref()) {
        (Some(image), Some(layout_handle)) => {
            let texture_count = atlas_layouts
                .get(layout_handle)
                .map(|layout| layout.textures.len())
                .unwrap_or(0);
            Some((image.clone(), layout_handle.clone(), texture_count))
        }
        _ => None,
    };
    let visible_layers = paper_doll.visible_layer_map();
    let palette_count = paper_doll.palette_catalog.palettes.len();
    let global_palette = global_palette_selection(&palette_panel, palette_count);

    for (marker, mut image, mut node) in &mut viewer_images {
        node.display = Display::None;
        let palette_selection =
            effective_layer_palette_selection(marker.layer, &layer_palettes, global_palette)
                .map(|(selection, _)| active_palette_selection(selection));

        if let Some((atlas_index, effective_flip)) = atlas_render {
            let mut rendered = false;

            if let Some(part_index) = visible_layers.get(&marker.layer).copied()
                && let Some(part) = paper_doll.catalog.parts.get(part_index)
                && let (Some(part_image), Some(part_layout)) = (
                    paper_doll.image_handles.get(&part.part_key),
                    paper_doll.atlas_layouts.get(&part.part_key),
                )
            {
                let texture_count = atlas_layouts
                    .get(part_layout)
                    .map(|layout| layout.textures.len())
                    .unwrap_or(0);
                if atlas_index < texture_count {
                    let palette_map = palette_selection.as_ref().and_then(|selection| {
                        palette_remap_map_for_part(
                            part,
                            selection,
                            &paper_doll,
                            &images,
                            &mut palette_remap_cache,
                        )
                    });
                    let display_handle = palette_selection.as_ref().map_or_else(
                        || part_image.clone(),
                        |selection| {
                            remapped_image_handle(
                                part_image,
                                &part.part_key,
                                selection,
                                palette_map.as_ref(),
                                &mut palette_remap_cache,
                                &mut images,
                            )
                        },
                    );
                    *image = ImageNode::from_atlas_image(
                        display_handle,
                        TextureAtlas {
                            layout: part_layout.clone(),
                            index: atlas_index,
                        },
                    );
                    image.flip_x = effective_flip;
                    node.display = Display::Flex;
                    rendered = true;
                }
            }

            if !rendered
                && marker.layer == LayerCode::Body01
                && let Some((sheet_image, layout_handle, texture_count)) = &body_atlas_data
                && atlas_index < *texture_count
            {
                *image = ImageNode::from_atlas_image(
                    sheet_image.clone(),
                    TextureAtlas {
                        layout: layout_handle.clone(),
                        index: atlas_index,
                    },
                );
                image.flip_x = effective_flip;
                node.display = Display::Flex;
                rendered = true;
            }

            if rendered {
                continue;
            }
        }
        *image = ImageNode::default();
    }
}

#[derive(Clone, Copy)]
struct ActivePaletteSelection {
    variant: usize,
}

fn active_palette_selection(selection: LayerPaletteSelection) -> ActivePaletteSelection {
    ActivePaletteSelection {
        variant: selection.variant,
    }
}

fn palette_remap_map_for_part(
    part: &crate::features::layers::PartDef,
    selection: &ActivePaletteSelection,
    paper_doll: &PaperDollState,
    images: &Assets<Image>,
    cache: &mut PaletteRemapCache,
) -> Option<HashMap<u32, [u8; 4]>> {
    let suffix = part
        .part_id
        .palette
        .or_else(|| inferred_palette_suffix_for_layer(part.part_id.layer))?;
    let cache_key = format!("{}::{}", suffix, selection.variant);
    if let Some(map) = cache.remap_tables.get(&cache_key) {
        return Some(map.clone());
    }

    let map = build_part_palette_remap_map(suffix, selection.variant, paper_doll, images)?;
    cache.remap_tables.insert(cache_key, map.clone());
    Some(map)
}

fn build_part_palette_remap_map(
    suffix: char,
    variant: usize,
    paper_doll: &PaperDollState,
    images: &Assets<Image>,
) -> Option<HashMap<u32, [u8; 4]>> {
    let mut map = HashMap::new();

    let specs: &[(&str, u32, usize, &str, u32, usize)] = match suffix {
        'a' => &[(
            "palettes/base ramps/3-color base ramp (00a)",
            0,
            4,
            "palettes/mana seed 3-color ramps",
            0,
            4,
        )],
        'b' => &[(
            "palettes/base ramps/4-color base ramp (00b)",
            0,
            5,
            "palettes/mana seed 4-color ramps",
            0,
            5,
        )],
        'c' => &[
            (
                "palettes/base ramps/2x 3-color base ramps (00c)",
                0,
                4,
                "palettes/mana seed 3-color ramps",
                0,
                4,
            ),
            (
                "palettes/base ramps/2x 3-color base ramps (00c)",
                8,
                4,
                "palettes/mana seed 3-color ramps",
                0,
                4,
            ),
        ],
        'd' => &[
            (
                "palettes/base ramps/4-color + 3-color base ramps (00d)",
                0,
                5,
                "palettes/mana seed 4-color ramps",
                0,
                5,
            ),
            (
                "palettes/base ramps/4-color + 3-color base ramps (00d)",
                10,
                4,
                "palettes/mana seed 3-color ramps",
                0,
                4,
            ),
        ],
        'f' => &[
            (
                "palettes/base ramps/4-color base ramp (00b)",
                0,
                5,
                "palettes/mana seed 4-color ramps",
                0,
                5,
            ),
            (
                "palettes/base ramps/skin color base ramp",
                0,
                5,
                "palettes/mana seed skin ramps",
                0,
                5,
            ),
            (
                "palettes/base ramps/hair color base ramp",
                0,
                6,
                "palettes/mana seed hair ramps",
                0,
                6,
            ),
        ],
        _ => return None,
    };

    for (source_key, source_x, source_count, target_key, target_x, target_count) in specs {
        let source_image = find_palette_image(paper_doll, images, source_key)?;
        let target_image = find_palette_image(paper_doll, images, target_key)?;

        let source_ramp = extract_ramp_variant(source_image, *source_x, *source_count, 0)?;
        let target_ramp = extract_ramp_variant(target_image, *target_x, *target_count, variant)?;
        for (from, to) in source_ramp.into_iter().zip(target_ramp.into_iter()) {
            map.insert(from, unpack_rgba8(to));
        }
    }

    if map.is_empty() { None } else { Some(map) }
}

fn inferred_palette_suffix_for_layer(layer: LayerCode) -> Option<char> {
    match layer {
        // Most body/hair sheets are version 00 without explicit palette suffix,
        // but they still use the standard body/hair ramp colors.
        LayerCode::Body01 | LayerCode::Hair13 => Some('f'),
        _ => None,
    }
}

fn find_palette_image<'a>(
    paper_doll: &'a PaperDollState,
    images: &'a Assets<Image>,
    key_tail: &str,
) -> Option<&'a Image> {
    let key_tail = key_tail.to_ascii_lowercase();
    for (key, handle) in &paper_doll.palette_image_handles {
        let normalized = key.replace('\\', "/").to_ascii_lowercase();
        if normalized.ends_with(&key_tail) {
            return images.get(handle);
        }
    }
    None
}

fn extract_ramp_variant(
    image: &Image,
    x_start: u32,
    count: usize,
    variant: usize,
) -> Option<Vec<u32>> {
    let block_width = 2u32;
    let block_height = 2u32;
    let variant_count = ((image.height() as usize) / (block_height as usize)).max(1);
    let variant = variant.min(variant_count.saturating_sub(1));

    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let x = x_start + (i as u32 * block_width) + (block_width / 2);
        let strip = extract_vertical_ramp(image, x, 0, block_height, variant_count)?;
        out.push(strip[variant]);
    }
    Some(out)
}

fn extract_vertical_ramp(
    image: &Image,
    x: u32,
    y_start: u32,
    block_height: u32,
    count: usize,
) -> Option<Vec<u32>> {
    let data = image.data.as_ref()?;
    let width = image.texture_descriptor.size.width;
    let height = image.texture_descriptor.size.height;
    if x >= width || block_height == 0 {
        return None;
    }

    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let y = y_start + (i as u32 * block_height) + (block_height / 2);
        if y >= height {
            return None;
        }
        let idx = ((y * width + x) * 4) as usize;
        if idx + 3 >= data.len() {
            return None;
        }
        out.push(pack_rgba8(
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3],
        ));
    }
    Some(out)
}

fn pack_rgba8(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

fn unpack_rgba8(value: u32) -> [u8; 4] {
    [
        ((value >> 24) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    ]
}

fn remapped_image_handle(
    source_handle: &Handle<Image>,
    source_key: &str,
    selection: &ActivePaletteSelection,
    palette_map: Option<&HashMap<u32, [u8; 4]>>,
    cache: &mut PaletteRemapCache,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    let Some(palette_map) = palette_map else {
        return source_handle.clone();
    };
    if palette_map.is_empty() {
        return source_handle.clone();
    }

    let cache_key = format!("{}::variant{}", source_key, selection.variant);
    if let Some(handle) = cache.remapped_handles.get(&cache_key) {
        return handle.clone();
    }

    let Some(source_image) = images.get(source_handle).cloned() else {
        return source_handle.clone();
    };
    let Some(remapped_image) = remap_image_colors(source_image, palette_map) else {
        return source_handle.clone();
    };
    let remapped_handle = images.add(remapped_image);
    cache
        .remapped_handles
        .insert(cache_key, remapped_handle.clone());
    remapped_handle
}

fn remap_image_colors(mut image: Image, palette_map: &HashMap<u32, [u8; 4]>) -> Option<Image> {
    let mut changed = false;
    let width = image.width();
    let height = image.height();
    for y in 0..height {
        for x in 0..width {
            let Some(pixel) = image.pixel_bytes_mut(UVec3::new(x, y, 0)) else {
                continue;
            };
            if pixel.len() < 4 {
                continue;
            }
            let key = rgba_key(pixel);
            if let Some(target) = palette_map.get(&key) {
                let alpha = pixel[3];
                pixel[0] = target[0];
                pixel[1] = target[1];
                pixel[2] = target[2];
                pixel[3] = alpha;
                changed = true;
            }
        }
    }
    if changed { Some(image) } else { None }
}

fn rgba_key(pixel: &[u8]) -> u32 {
    if pixel.len() < 4 {
        return 0;
    }
    pack_rgba8(pixel[0], pixel[1], pixel[2], pixel[3])
}

pub(super) fn sync_anim_viewer_button_styles(
    anim_state: Res<AnimationAuthoringState>,
    viewer: Res<AnimViewerState>,
    mut button_sets: ParamSet<(
        Query<(
            &ViewerDirectionButtonStyleMarker,
            &mut BorderColor,
            &mut BackgroundColor,
        )>,
        Query<(
            &ViewerSpeedButtonStyleMarker,
            &mut BorderColor,
            &mut BackgroundColor,
        )>,
        Query<(&mut BorderColor, &mut BackgroundColor), With<ViewerPlayPauseButton>>,
        Query<(&mut BorderColor, &mut BackgroundColor), With<ViewerLoopOverrideButton>>,
        Query<
            (&mut BorderColor, &mut BackgroundColor),
            Or<(With<ViewerPrevFrameButton>, With<ViewerNextFrameButton>)>,
        >,
        Query<&mut Text, With<ViewerPlayPauseLabel>>,
        Query<&mut Text, With<ViewerLoopOverrideLabel>>,
    )>,
) {
    if !anim_state.is_changed() && !viewer.is_changed() {
        return;
    }

    let left_enabled = anim_state
        .active_clip()
        .is_some_and(viewer_left_direction_available);

    for (marker, mut border, mut bg) in &mut button_sets.p0() {
        let is_left = marker.direction == Direction::Left;
        let enabled = !is_left || left_enabled;
        let active = viewer.dir == marker.direction;
        *bg = BackgroundColor(if !enabled {
            Color::srgb(0.08, 0.08, 0.10)
        } else if active {
            Color::srgb(0.34, 0.30, 0.12)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
        *border = BorderColor::all(if !enabled {
            Color::srgba(1.0, 1.0, 1.0, 0.08)
        } else if active {
            Color::srgb(0.95, 0.82, 0.22)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.20)
        });
    }

    for (marker, mut border, mut bg) in &mut button_sets.p1() {
        let active = (viewer.speed - marker.speed).abs() < 0.05;
        *bg = BackgroundColor(if active {
            Color::srgb(0.24, 0.22, 0.14)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
        *border = BorderColor::all(if active {
            Color::srgb(0.95, 0.82, 0.22)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.20)
        });
    }

    for mut text in &mut button_sets.p5() {
        *text = Text::new(if viewer.is_playing { "Pause" } else { "Play" });
    }
    for mut text in &mut button_sets.p6() {
        *text = Text::new(if viewer.loop_override { "On" } else { "Off" });
    }

    for (mut border, mut bg) in &mut button_sets.p2() {
        *bg = BackgroundColor(if viewer.is_playing {
            Color::srgb(0.24, 0.22, 0.14)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
        *border = BorderColor::all(if viewer.is_playing {
            Color::srgb(0.95, 0.82, 0.22)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.20)
        });
    }
    for (mut border, mut bg) in &mut button_sets.p3() {
        *bg = BackgroundColor(if viewer.loop_override {
            Color::srgb(0.24, 0.22, 0.14)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
        *border = BorderColor::all(if viewer.loop_override {
            Color::srgb(0.95, 0.82, 0.22)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.20)
        });
    }

    for (mut border, mut bg) in &mut button_sets.p4() {
        *bg = BackgroundColor(if viewer.is_playing {
            Color::srgb(0.12, 0.12, 0.15)
        } else {
            Color::srgb(0.16, 0.16, 0.20)
        });
        *border = BorderColor::all(if viewer.is_playing {
            Color::srgba(1.0, 1.0, 1.0, 0.10)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.20)
        });
    }
}

pub(super) fn mark_preview_strip_dirty(
    anim_state: Res<AnimationAuthoringState>,
    panel_state: Res<AnimationPanelState>,
    grid: Res<GridState>,
    loaded: Res<LoadedImage>,
    preview_atlas: Res<PreviewAtlasState>,
    mut preview_strip: ResMut<PreviewStripUiState>,
) {
    if anim_state.is_changed()
        || panel_state.is_changed()
        || grid.is_changed()
        || loaded.is_changed()
        || preview_atlas.is_changed()
    {
        preview_strip.dirty = true;
    }
}

pub(super) fn sync_preview_atlas_layout(
    loaded: Res<LoadedImage>,
    grid: Res<GridState>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut preview_atlas: ResMut<PreviewAtlasState>,
    mut preview_strip: ResMut<PreviewStripUiState>,
) {
    let Some(image_handle) = loaded.handle.clone() else {
        if preview_atlas.image.take().is_some() || preview_atlas.layout.take().is_some() {
            preview_strip.dirty = true;
        }
        return;
    };

    let columns = grid.columns.max(1);
    let rows = derive_preview_atlas_rows(loaded.size, &grid);
    let cell_size = UVec2::new(grid.cell_width.max(1), grid.cell_height.max(1));
    let spacing = UVec2::ZERO;
    let offset = UVec2::new(grid.offset_x, grid.offset_y);

    let needs_rebuild = preview_atlas.image.as_ref() != Some(&image_handle)
        || preview_atlas.layout.is_none()
        || preview_atlas.columns != columns
        || preview_atlas.rows != rows
        || preview_atlas.cell_size != cell_size
        || preview_atlas.spacing != spacing
        || preview_atlas.offset != offset;
    if !needs_rebuild {
        return;
    }

    let layout =
        TextureAtlasLayout::from_grid(cell_size, columns, rows, Some(spacing), Some(offset));
    let layout_handle = atlas_layouts.add(layout);
    preview_atlas.image = Some(image_handle);
    preview_atlas.layout = Some(layout_handle);
    preview_atlas.columns = columns;
    preview_atlas.rows = rows;
    preview_atlas.cell_size = cell_size;
    preview_atlas.spacing = spacing;
    preview_atlas.offset = offset;
    preview_strip.dirty = true;
}

pub(super) fn rebuild_preview_strip_if_needed(
    mut commands: Commands,
    mut preview_strip: ResMut<PreviewStripUiState>,
    preview_content: Single<Entity, With<PreviewStripContent>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    preview_atlas: Res<PreviewAtlasState>,
    anim_state: Res<AnimationAuthoringState>,
    panel_state: Res<AnimationPanelState>,
) {
    if !preview_strip.dirty {
        return;
    }

    for entity in preview_strip.dynamic_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    if panel_state.active_dir.is_none() {
        preview_strip.dirty = false;
        return;
    }

    let Some(track) = anim_state.active_track() else {
        preview_strip.dirty = false;
        return;
    };

    let mut spawned_entities = Vec::new();
    let atlas_data = match (preview_atlas.image.as_ref(), preview_atlas.layout.as_ref()) {
        (Some(image), Some(layout_handle)) => {
            let texture_count = atlas_layouts
                .get(layout_handle)
                .map(|layout| layout.textures.len())
                .unwrap_or(0);
            Some((image.clone(), layout_handle.clone(), texture_count))
        }
        _ => None,
    };
    commands.entity(*preview_content).with_children(|content| {
        for (step_index, step) in track.steps.iter().enumerate() {
            let is_selected = panel_state.selected_steps.contains(&step_index);
            let entity = content
                .spawn((
                    Button,
                    Node {
                        width: px(78),
                        height: px(78),
                        padding: UiRect::all(px(4)),
                        border: UiRect::all(px(2)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                    BorderColor::all(if is_selected {
                        Color::srgb(0.95, 0.82, 0.22)
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.20)
                    }),
                    PreviewStepButton { step_index },
                ))
                .with_children(|tile| {
                    tile.spawn((
                        Node {
                            width: px(66),
                            height: px(66),
                            border: UiRect::all(px(1)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.08, 0.08, 0.10)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
                    ))
                    .with_children(|thumb| {
                        let atlas_index = step.cell as usize;
                        if let Some((image_handle, atlas_layout_handle, texture_count)) =
                            &atlas_data
                            && atlas_index < *texture_count
                        {
                            let mut image = ImageNode::from_atlas_image(
                                image_handle.clone(),
                                TextureAtlas {
                                    layout: atlas_layout_handle.clone(),
                                    index: atlas_index,
                                },
                            );
                            image.flip_x = step.flip_x;
                            thumb.spawn((
                                image,
                                Node {
                                    width: px(62),
                                    height: px(62),
                                    ..default()
                                },
                            ));
                        } else {
                            thumb.spawn((
                                Text::new(step.cell.to_string()),
                                TextFont::from_font_size(9.0),
                            ));
                        }
                    });
                    let label = format!("#{} {}", step_index, step.cell);
                    tile.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(2),
                            left: px(2),
                            padding: UiRect::axes(px(3), px(1)),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.06, 0.06, 0.08, 0.85)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.24)),
                    ))
                    .with_children(|overlay| {
                        overlay.spawn((Text::new(label), TextFont::from_font_size(9.0)));
                    });
                })
                .id();
            spawned_entities.push(entity);
        }
    });
    preview_strip.dynamic_entities = spawned_entities;
    preview_strip.dirty = false;
}

pub(super) fn sync_animation_field_drafts_from_state(
    anim_state: &AnimationAuthoringState,
    panel_state: &AnimationPanelState,
    drafts: &mut AnimationFieldDrafts,
) {
    let selected_step = panel_state
        .selected_steps
        .iter()
        .next()
        .copied()
        .or(anim_state.active_step);
    if panel_state.active_dir.is_some() {
        if let Some(step_index) = selected_step {
            if let Some(step) = anim_state
                .active_track()
                .and_then(|track| track.steps.get(step_index))
            {
                drafts.step_ms = step.ms.to_string();
            }
        }
    }

    if let Some(clip) = anim_state.active_clip() {
        match clip.playback {
            Playback::Loop => {}
            Playback::LoopN { times } => drafts.loop_n_times = times.to_string(),
            Playback::OneShot { hold_last } => drafts.hold_last = hold_last.to_string(),
        }
    }
}

pub(super) fn sync_animation_text(
    anim_state: Res<AnimationAuthoringState>,
    panel_state: Res<AnimationPanelState>,
    mut animation_fields: ResMut<AnimationFieldDrafts>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<ActiveAnimationText>>,
        Query<&mut Text, With<ActiveTrackText>>,
        Query<&mut Text, With<PlaybackText>>,
    )>,
) {
    if !anim_state.is_changed() && !panel_state.is_changed() {
        return;
    }

    sync_animation_field_drafts_from_state(&anim_state, &panel_state, &mut animation_fields);

    let Some(clip) = anim_state.active_clip() else {
        return;
    };
    let track = if panel_state.active_dir.is_some() {
        anim_state.active_track()
    } else {
        None
    };
    let step_count = track.map(|t| t.steps.len()).unwrap_or(0);
    let dir_label = panel_state
        .active_dir
        .map(|direction| direction.label().to_string())
        .unwrap_or_else(|| "View".to_string());

    let active_line = format!(
        "Animations > {} > {} > {}",
        clip.category, clip.display, dir_label
    );
    let track_line = if panel_state.active_dir == Some(Direction::Left) {
        format!(
            "Status: Viewing (Left derived from Right), selected steps: {}",
            panel_state.selected_steps.len()
        )
    } else if is_edit_armed(panel_state.active_dir) {
        format!(
            "Status: Editing: {} / {}, selected steps: {}",
            clip.display,
            panel_state.active_dir.unwrap_or(Direction::Up).label(),
            panel_state.selected_steps.len()
        )
    } else {
        "Status: Viewing".to_string()
    };
    let playback_mode = match clip.playback {
        Playback::Loop => "Loop".to_string(),
        Playback::LoopN { times } => format!("Loop N ({times})"),
        Playback::OneShot { hold_last } => format!("One Shot (hold_last={hold_last})"),
    };
    let playback_line = format!("Frames: {step_count} | Playback: {playback_mode}");

    for mut text in &mut text_sets.p0() {
        *text = Text::new(active_line.clone());
    }
    for mut text in &mut text_sets.p1() {
        *text = Text::new(track_line.clone());
    }
    for mut text in &mut text_sets.p2() {
        *text = Text::new(playback_line.clone());
    }
}

pub(super) fn sync_animation_field_widgets(
    drafts: Res<AnimationFieldDrafts>,
    mut field_texts: Query<(&AnimationFieldText, &mut Text)>,
    mut field_borders: Query<(&AnimationFieldButton, &mut BorderColor)>,
) {
    if !drafts.is_changed() {
        return;
    }
    for (field_text, mut text) in &mut field_texts {
        *text = Text::new(drafts.value(field_text.field));
    }
    for (field_button, mut border) in &mut field_borders {
        let color = if drafts.is_invalid(field_button.field) {
            Color::srgb(0.82, 0.18, 0.18)
        } else if drafts.active == Some(field_button.field) {
            Color::srgb(0.34, 0.56, 0.98)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.22)
        };
        *border = BorderColor::all(color);
    }
}

pub(super) fn sync_animation_project_sheet(
    grid: Res<GridState>,
    loaded: Res<LoadedImage>,
    mut anim_state: ResMut<AnimationAuthoringState>,
) {
    if !grid.is_changed() && !loaded.is_changed() {
        return;
    }
    anim_state.sync_sheet_meta(&grid, &loaded);
}

pub(super) fn sync_workspace_highlights_derived(
    anim_state: Res<AnimationAuthoringState>,
    panel_state: Res<AnimationPanelState>,
    mut selection_state: ResMut<SelectionState>,
) {
    if !anim_state.is_changed() && !panel_state.is_changed() {
        return;
    }
    sync_workspace_selection_from_track(&anim_state, &panel_state, &mut selection_state);
}

pub(super) fn parse_bool_input(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Some(true),
        "false" | "0" | "no" | "n" => Some(false),
        _ => None,
    }
}

pub(super) fn derive_preview_atlas_rows(image_size: Option<UVec2>, grid: &GridState) -> u32 {
    let fallback = grid.rows.max(1);
    let Some(image_size) = image_size else {
        return fallback;
    };
    let cell_height = grid.cell_height.max(1);
    if image_size.y <= grid.offset_y {
        return fallback;
    }
    let available = image_size.y - grid.offset_y;
    (available / cell_height).max(1)
}

pub(super) fn reset_viewer_playback(viewer: &mut AnimViewerState) {
    viewer.step_index = 0;
    viewer.step_elapsed_ms = 0.0;
    viewer.loop_n_remaining = None;
}

pub(super) struct ResolvedViewerTrack<'a> {
    pub(super) track: &'a Track,
    pub(super) force_flip_x: bool,
}

pub(super) fn resolve_viewer_track(
    clip: &Clip,
    desired: Direction,
) -> Option<ResolvedViewerTrack<'_>> {
    if let Some(track) = clip.tracks.get(&desired) {
        return Some(ResolvedViewerTrack {
            track,
            force_flip_x: false,
        });
    }

    if desired == Direction::Left
        && clip_derives_left_from_right(clip)
        && let Some(track) = clip.tracks.get(&Direction::Right)
    {
        return Some(ResolvedViewerTrack {
            track,
            force_flip_x: true,
        });
    }

    for fallback in [
        Direction::Down,
        Direction::Right,
        Direction::Up,
        Direction::Left,
    ] {
        if fallback == desired {
            continue;
        }
        if let Some(track) = clip.tracks.get(&fallback) {
            return Some(ResolvedViewerTrack {
                track,
                force_flip_x: false,
            });
        }
        if fallback == Direction::Left
            && clip_derives_left_from_right(clip)
            && let Some(track) = clip.tracks.get(&Direction::Right)
        {
            return Some(ResolvedViewerTrack {
                track,
                force_flip_x: true,
            });
        }
    }

    None
}

pub(super) fn clip_derives_left_from_right(clip: &Clip) -> bool {
    clip.derives.iter().any(|rule| {
        matches!(
            rule,
            DeriveRule::MirrorX {
                from: Direction::Right,
                to: Direction::Left
            }
        )
    })
}

pub(super) fn viewer_left_direction_available(clip: &Clip) -> bool {
    clip.tracks.contains_key(&Direction::Left)
        || (clip_derives_left_from_right(clip) && clip.tracks.contains_key(&Direction::Right))
}

pub(super) fn is_edit_armed(active_dir: Option<Direction>) -> bool {
    active_dir.is_some_and(Direction::is_authored)
}

pub(super) fn sync_workspace_selection_from_track(
    anim_state: &AnimationAuthoringState,
    panel_state: &AnimationPanelState,
    selection_state: &mut SelectionState,
) {
    selection_state.selected_cells.clear();
    if panel_state.active_dir.is_none() {
        return;
    }
    for cell in anim_state.active_track_unique_cells() {
        selection_state.selected_cells.insert(cell);
    }
}

pub(super) fn sync_cell_visuals(
    selection_state: Res<SelectionState>,
    mut cells: Query<(&GridCellButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    if !selection_state.is_changed() {
        return;
    }
    for (cell, mut bg, mut border) in &mut cells {
        let selected = selection_state.selected_cells.contains(&cell.index);
        *bg = BackgroundColor(if selected {
            overlay::selected_fill_color()
        } else {
            Color::NONE
        });
        *border = BorderColor::all(if selected {
            overlay::selected_border_color()
        } else {
            overlay::grid_line_color()
        });
    }
}
