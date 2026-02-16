use super::*;
use bevy::ecs::system::SystemParam;

type PanelScrollSet<'w, 's> = ParamSet<
    'w,
    's,
    (
        Query<
            'w,
            's,
            (&'static mut ScrollPosition, &'static ComputedNode),
            With<LeftPanelInputRegion>,
        >,
        Query<
            'w,
            's,
            (&'static mut ScrollPosition, &'static ComputedNode),
            With<RightPanelInputRegion>,
        >,
        Query<
            'w,
            's,
            (&'static mut ScrollPosition, &'static ComputedNode),
            With<BottomPanelInputRegion>,
        >,
    ),
>;

type CanvasViewportRef<'w, 's> =
    Single<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<CanvasViewport>>;

#[derive(SystemParam)]
pub(super) struct WheelInputCtx<'w, 's> {
    mouse_wheel_reader: MessageReader<'w, 's, MouseWheel>,
    keyboard_input: Res<'w, ButtonInput<KeyCode>>,
    input_context: Res<'w, InputContext>,
    panel_scrolls: PanelScrollSet<'w, 's>,
    viewport: CanvasViewportRef<'w, 's>,
    window: Single<'w, 's, &'static Window>,
    grid: Res<'w, GridState>,
    loaded: Res<'w, LoadedImage>,
    canvas_view: ResMut<'w, CanvasView>,
    canvas_ui: ResMut<'w, CanvasUiState>,
}

#[derive(SystemParam)]
pub(super) struct CanvasInteractionCtx<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    window: Single<'w, 's, &'static Window>,
    viewport: CanvasViewportRef<'w, 's>,
    input_context: Res<'w, InputContext>,
    loaded: ResMut<'w, LoadedImage>,
    anim_state: ResMut<'w, AnimationAuthoringState>,
    panel_state: Res<'w, AnimationPanelState>,
    grid: Res<'w, GridState>,
    canvas_view: ResMut<'w, CanvasView>,
    pan_state: ResMut<'w, CanvasPanState>,
    selection_state: ResMut<'w, SelectionState>,
    canvas_ui: ResMut<'w, CanvasUiState>,
}

pub(super) fn update_hovered_region(
    left_region: Single<&RelativeCursorPosition, With<LeftPanelInputRegion>>,
    canvas_region: Single<&RelativeCursorPosition, With<CanvasViewport>>,
    right_region: Single<&RelativeCursorPosition, With<RightPanelInputRegion>>,
    bottom_region: Single<&RelativeCursorPosition, With<BottomPanelInputRegion>>,
    mut input_context: ResMut<InputContext>,
) {
    input_context.hovered_region = if canvas_region.cursor_over() {
        Some(InputRegion::GridCanvas)
    } else if left_region.cursor_over() {
        Some(InputRegion::LeftPanel)
    } else if right_region.cursor_over() {
        Some(InputRegion::RightPanel)
    } else if bottom_region.cursor_over() {
        Some(InputRegion::BottomPanel)
    } else {
        None
    };
}

pub(super) fn update_input_capture(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input_context: ResMut<InputContext>,
) {
    let pan_modifier = keyboard.pressed(KeyCode::Space);
    let pan_start = mouse.just_pressed(MouseButton::Middle)
        || (pan_modifier && mouse.just_pressed(MouseButton::Left));
    if pan_start && input_context.hovered_region == Some(InputRegion::GridCanvas) {
        input_context.active_capture = Some(InputRegion::GridCanvas);
    }

    let pan_end =
        mouse.just_released(MouseButton::Middle) || mouse.just_released(MouseButton::Left);
    let still_panning =
        mouse.pressed(MouseButton::Middle) || (pan_modifier && mouse.pressed(MouseButton::Left));
    if pan_end && !still_panning {
        input_context.active_capture = None;
    }
}

pub(super) fn route_wheel_input(mut ctx: WheelInputCtx) {
    let target = ctx
        .input_context
        .active_capture
        .or(ctx.input_context.hovered_region);
    let Some(target) = target else {
        for _ in ctx.mouse_wheel_reader.read() {}
        return;
    };

    let (viewport_node, viewport_transform) = *ctx.viewport;
    let viewport_logical_size = viewport_node.size() * viewport_node.inverse_scale_factor();
    let local_cursor = viewport_cursor_position(&ctx.window, viewport_node, viewport_transform)
        .unwrap_or(viewport_logical_size * 0.5);

    for mouse_wheel in ctx.mouse_wheel_reader.read() {
        match target {
            InputRegion::LeftPanel => {
                let mut left_query = ctx.panel_scrolls.p0();
                let Ok((mut scroll_position, scroll_content)) = left_query.single_mut() else {
                    continue;
                };
                let (mut dx, mut dy) = match mouse_wheel.unit {
                    MouseScrollUnit::Line => (mouse_wheel.x * 20.0, mouse_wheel.y * 20.0),
                    MouseScrollUnit::Pixel => (mouse_wheel.x, mouse_wheel.y),
                };
                if ctx.keyboard_input.pressed(KeyCode::ShiftLeft)
                    || ctx.keyboard_input.pressed(KeyCode::ShiftRight)
                {
                    std::mem::swap(&mut dx, &mut dy);
                }
                apply_panel_scroll(&mut scroll_position, scroll_content, dx, dy);
            }
            InputRegion::RightPanel => {
                let mut right_query = ctx.panel_scrolls.p1();
                let Ok((mut scroll_position, scroll_content)) = right_query.single_mut() else {
                    continue;
                };
                let (mut dx, mut dy) = match mouse_wheel.unit {
                    MouseScrollUnit::Line => (mouse_wheel.x * 20.0, mouse_wheel.y * 20.0),
                    MouseScrollUnit::Pixel => (mouse_wheel.x, mouse_wheel.y),
                };
                if ctx.keyboard_input.pressed(KeyCode::ShiftLeft)
                    || ctx.keyboard_input.pressed(KeyCode::ShiftRight)
                {
                    std::mem::swap(&mut dx, &mut dy);
                }
                apply_panel_scroll(&mut scroll_position, scroll_content, dx, dy);
            }
            InputRegion::BottomPanel => {
                let mut bottom_query = ctx.panel_scrolls.p2();
                let Ok((mut scroll_position, scroll_content)) = bottom_query.single_mut() else {
                    continue;
                };
                let (mut dx, mut dy) = match mouse_wheel.unit {
                    MouseScrollUnit::Line => (mouse_wheel.x * 20.0, mouse_wheel.y * 20.0),
                    MouseScrollUnit::Pixel => (mouse_wheel.x, mouse_wheel.y),
                };
                if ctx.keyboard_input.pressed(KeyCode::ShiftLeft)
                    || ctx.keyboard_input.pressed(KeyCode::ShiftRight)
                {
                    std::mem::swap(&mut dx, &mut dy);
                }
                apply_panel_scroll(&mut scroll_position, scroll_content, dx, dy);
            }
            InputRegion::GridCanvas => {
                let delta = match mouse_wheel.unit {
                    MouseScrollUnit::Line => mouse_wheel.y * 0.1,
                    MouseScrollUnit::Pixel => mouse_wheel.y * 0.001,
                };
                if delta.abs() <= f32::EPSILON {
                    continue;
                }

                let old_zoom = ctx.canvas_view.zoom;
                let new_zoom = (old_zoom * (1.0 + delta)).clamp(0.25, 4.0);
                if (new_zoom - old_zoom).abs() <= f32::EPSILON {
                    continue;
                }

                let world_under_cursor = (local_cursor - ctx.canvas_view.offset) / old_zoom;
                ctx.canvas_view.zoom = new_zoom;
                ctx.canvas_view.offset = local_cursor - world_under_cursor * new_zoom;
                let content_size =
                    canvas_content_size(&ctx.grid, &ctx.loaded) * ctx.canvas_view.zoom;
                ctx.canvas_view.offset = clamp_canvas_offset(
                    ctx.canvas_view.offset,
                    content_size,
                    viewport_logical_size,
                );
                ctx.canvas_ui.dirty = true;
            }
        }
    }
}

pub(super) fn handle_canvas_interaction(ctx: CanvasInteractionCtx) {
    let CanvasInteractionCtx {
        mouse,
        keyboard,
        window,
        viewport,
        input_context,
        mut loaded,
        mut anim_state,
        panel_state,
        grid,
        mut canvas_view,
        mut pan_state,
        mut selection_state,
        mut canvas_ui,
    } = ctx;

    let (viewport_node, viewport_transform) = *viewport;
    let viewport_logical_size = viewport_node.size() * viewport_node.inverse_scale_factor();
    let target = input_context
        .active_capture
        .or(input_context.hovered_region);
    let pan_modifier = keyboard.pressed(KeyCode::Space);
    let pan_buttons_down =
        mouse.pressed(MouseButton::Middle) || (pan_modifier && mouse.pressed(MouseButton::Left));

    if pan_buttons_down && target == Some(InputRegion::GridCanvas) {
        if let Some(cursor_pos) = window.cursor_position() {
            if pan_state.dragging
                && let Some(last_cursor) = pan_state.last_window_cursor
            {
                canvas_view.offset += cursor_pos - last_cursor;
                let content_size = canvas_content_size(&grid, &loaded) * canvas_view.zoom;
                canvas_view.offset =
                    clamp_canvas_offset(canvas_view.offset, content_size, viewport_logical_size);
                canvas_ui.dirty = true;
            }
            pan_state.dragging = true;
            pan_state.last_window_cursor = Some(cursor_pos);
        }
    } else {
        pan_state.dragging = false;
        pan_state.last_window_cursor = None;
    }

    if pan_state.dragging || pan_modifier || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if target != Some(InputRegion::GridCanvas) {
        return;
    }
    if !is_edit_armed(panel_state.active_dir) {
        return;
    }

    let Some(local) = viewport_cursor_position(&window, viewport_node, viewport_transform) else {
        return;
    };
    let world = (local - canvas_view.offset) / canvas_view.zoom;
    if world.x < grid.offset_x as f32 || world.y < grid.offset_y as f32 {
        return;
    }

    let col = ((world.x - grid.offset_x as f32) / grid.cell_width as f32).floor() as u32;
    let row = ((world.y - grid.offset_y as f32) / grid.cell_height as f32).floor() as u32;
    if row >= grid.rows || col >= grid.columns {
        return;
    }

    let index = grid_state::cell_index(row, col, grid.columns);
    let Some(active_dir) = panel_state.active_dir else {
        return;
    };
    anim_state.active_direction = active_dir;
    match anim_state.toggle_cell_in_active_track(index) {
        Some(CellToggleResult::Added) => {
            loaded.status = Some(format!("Added frame cell {index}"));
        }
        Some(CellToggleResult::RemovedLast) => {
            loaded.status = Some(format!("Removed last frame cell {index}"));
        }
        None => {
            loaded.status = Some("Unable to edit active direction".to_string());
        }
    }
    sync_workspace_selection_from_track(&anim_state, &panel_state, &mut selection_state);
}

pub(super) fn viewport_cursor_position(
    window: &Window,
    viewport_node: &ComputedNode,
    viewport_transform: &UiGlobalTransform,
) -> Option<Vec2> {
    // UiGlobalTransform, contains_point, and normalize_point all operate in
    // physical-pixel space, so we must feed them the physical cursor position
    // (same as Bevy's own focus/picking system does).
    let physical_cursor = window.physical_cursor_position()?;
    if !viewport_node.contains_point(*viewport_transform, physical_cursor) {
        return None;
    }

    let normalized_centered =
        viewport_node.normalize_point(*viewport_transform, physical_cursor)?;
    let normalized = normalized_centered + Vec2::splat(0.5);
    // normalize_point returns [-0.5, 0.5] and size() is in physical pixels.
    // Multiply then convert to logical so the result matches the coordinate
    // space used by canvas_view.offset, grid sizes, etc.
    let physical_local = normalized * viewport_node.size();
    Some(physical_local * viewport_node.inverse_scale_factor())
}

pub(super) fn canvas_content_size(grid: &GridState, loaded: &LoadedImage) -> Vec2 {
    let image_size = loaded
        .size
        .map(|size| Vec2::new(size.x as f32, size.y as f32))
        .unwrap_or(Vec2::ZERO);
    let grid_size = overlay::grid_dimensions(grid);
    Vec2::new(
        (grid.offset_x as f32 + image_size.x.max(grid_size.x)).max(1.0),
        (grid.offset_y as f32 + image_size.y.max(grid_size.y)).max(1.0),
    )
}

pub(super) fn clamp_canvas_offset(offset: Vec2, content_size: Vec2, viewport_size: Vec2) -> Vec2 {
    let min_x = (viewport_size.x - content_size.x).min(0.0);
    let min_y = (viewport_size.y - content_size.y).min(0.0);
    Vec2::new(offset.x.clamp(min_x, 0.0), offset.y.clamp(min_y, 0.0))
}

pub(super) fn apply_panel_scroll(
    scroll_position: &mut ScrollPosition,
    scroll_content: &ComputedNode,
    dx: f32,
    dy: f32,
) {
    let visible_size = scroll_content.size();
    let content_size = scroll_content.content_size();
    let inverse = scroll_content.inverse_scale_factor;

    let x_range = (content_size.x - visible_size.x).max(0.0) * inverse;
    let y_range = (content_size.y - visible_size.y).max(0.0) * inverse;

    scroll_position.x = (scroll_position.x - dx).clamp(0.0, x_range);
    scroll_position.y = (scroll_position.y - dy).clamp(0.0, y_range);
}

pub(super) fn handle_splitter_press(
    splitters_query: Query<(&Interaction, &SplitterHandle), (Changed<Interaction>, With<Button>)>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut drag_state: ResMut<DragState>,
) {
    if mouse.just_released(MouseButton::Left) {
        drag_state.active = None;
        drag_state.last_cursor = None;
        return;
    }

    for (interaction, handle) in &splitters_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(cursor) = window.cursor_position() else {
            continue;
        };
        drag_state.active = Some(handle.kind);
        drag_state.last_cursor = Some(cursor);
    }
}

pub(super) fn apply_splitter_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut drag_state: ResMut<DragState>,
    mut layout: ResMut<LayoutState>,
) {
    let Some(active) = drag_state.active else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        drag_state.active = None;
        drag_state.last_cursor = None;
        return;
    }

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Some(last_cursor) = drag_state.last_cursor else {
        drag_state.last_cursor = Some(cursor);
        return;
    };

    let delta = cursor - last_cursor;
    match active {
        SplitterKind::Left => {
            layout.left_panel_width =
                (layout.left_panel_width + delta.x).max(LayoutState::MIN_SIDE_PANEL_WIDTH);
        }
        SplitterKind::Right => {
            layout.right_panel_width =
                (layout.right_panel_width - delta.x).max(LayoutState::MIN_SIDE_PANEL_WIDTH);
        }
        SplitterKind::Bottom => {
            layout.bottom_panel_height =
                (layout.bottom_panel_height - delta.y).max(LayoutState::MIN_BOTTOM_PANEL_HEIGHT);
        }
    }
    drag_state.last_cursor = Some(cursor);
}

pub(super) fn update_panel_sizes(
    layout: Res<LayoutState>,
    mut node_queries: ParamSet<(
        Query<&mut Node, With<LeftPanelWidthNode>>,
        Query<&mut Node, With<RightPanelWidthNode>>,
        Query<&mut Node, With<BottomPanelHeightNode>>,
    )>,
) {
    for mut node in &mut node_queries.p0() {
        node.width = px(layout.left_panel_width);
    }
    for mut node in &mut node_queries.p1() {
        node.width = px(layout.right_panel_width);
    }
    for mut node in &mut node_queries.p2() {
        node.height = px(layout.bottom_panel_height);
    }
}
