use super::*;
use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub(super) struct SetupUiCtx<'w, 's> {
    grid: Res<'w, GridState>,
    layout: Res<'w, LayoutState>,
    mode: Res<'w, EditorMode>,
    anim_state: Res<'w, AnimationAuthoringState>,
    outfits: Res<'w, OutfitDbState>,
    viewer_state: ResMut<'w, AnimViewerState>,
    grid_drafts: ResMut<'w, GridFieldDrafts>,
    animation_fields: ResMut<'w, AnimationFieldDrafts>,
    outfit_fields: ResMut<'w, OutfitFieldDrafts>,
    animation_panel: ResMut<'w, AnimationPanelState>,
    preview_strip: ResMut<'w, PreviewStripUiState>,
    _scratch: Local<'s, ()>,
}

pub(super) fn setup_ui_shell(mut commands: Commands, ui: SetupUiCtx) {
    let SetupUiCtx {
        grid,
        layout,
        mode,
        anim_state,
        outfits,
        mut viewer_state,
        mut grid_drafts,
        mut animation_fields,
        mut outfit_fields,
        mut animation_panel,
        mut preview_strip,
        ..
    } = ui;

    grid_drafts.set_from_grid(&grid);
    animation_fields.set_defaults();
    if let Some(clip) = anim_state.active_clip() {
        animation_panel.selected_category = Some(clip.category.clone());
        animation_panel
            .expanded_categories
            .insert(clip.category.clone());
    }
    animation_panel.active_dir = None;
    animation_panel.selected_steps.clear();
    preview_strip.dirty = true;
    viewer_state.clip_id = anim_state.active_clip().map(|clip| clip.id.clone());
    reset_viewer_playback(&mut viewer_state);
    if let Some(outfit) = outfits
        .selected
        .and_then(|index| outfits.db.outfits.get(index))
    {
        outfit_fields.set_from_outfit(outfit);
    } else {
        outfit_fields.clear_for_none_selected();
    }
    commands.spawn(Camera2d);

    commands
        .spawn((
            Name::new("RootUi"),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.08)),
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("TopToolbar"),
                top_toolbar::toolbar_node(layout.toolbar_height),
                top_toolbar::toolbar_bg(),
                GlobalZIndex(50),
            ))
            .with_children(|toolbar_row| {
                spawn_button(toolbar_row, "File", FileMenuToggleButton);
                toolbar_row.spawn((Text::new("Edit (Future)"),));
                spawn_button(toolbar_row, "Mode", ModeMenuToggleButton);
                toolbar_row.spawn((Text::new(top_toolbar::mode_text(*mode)), ModeTextLabel));

                toolbar_row
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(layout.toolbar_height - 1.0),
                            left: px(12),
                            width: px(190),
                            flex_direction: FlexDirection::Column,
                            display: Display::None,
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.11, 0.11, 0.14)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        FileMenuPanel,
                    ))
                    .with_children(|file_menu| {
                        spawn_button(file_menu, "Open Image", FileOpenMenuItem);
                        spawn_button(file_menu, "Load Anim Project", FileLoadProjectMenuItem);
                        spawn_button(file_menu, "Save Anim Project", FileSaveMenuItem);
                        file_menu
                            .spawn((
                                Node {
                                    padding: UiRect::axes(px(8), px(6)),
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.13, 0.13, 0.16)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16)),
                                FileExitMenuItem,
                            ))
                            .with_children(|button| {
                                button.spawn((Text::new("Exit"),));
                            });
                    });

                toolbar_row
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(layout.toolbar_height - 1.0),
                            left: px(150),
                            width: px(220),
                            flex_direction: FlexDirection::Column,
                            display: Display::None,
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.11, 0.11, 0.14)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ModeMenuPanel,
                    ))
                    .with_children(|mode_menu| {
                        spawn_button(mode_menu, "Animations", ModeAnimationsMenuItem);
                        spawn_button(mode_menu, "Parts", ModePartsMenuItem);
                        spawn_button(mode_menu, "Outfits", ModeOutfitsMenuItem);
                    });
            });

            root.spawn((
                Name::new("MainArea"),
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
            ))
            .with_children(|main_area| {
                main_area
                    .spawn((
                        Name::new("LeftPanel"),
                        left_panel::node(layout.left_panel_width),
                        left_panel::background(),
                        LeftPanelWidthNode,
                    ))
                    .with_children(|left| {
                        left.spawn(Node {
                            display: Display::Grid,
                            width: percent(100),
                            height: percent(100),
                            grid_template_columns: vec![
                                RepeatedGridTrack::flex(1, 1.0),
                                RepeatedGridTrack::auto(1),
                            ],
                            ..default()
                        })
                        .with_children(|left_frame| {
                            let scroll_id = left_frame
                                .spawn((
                                    Node {
                                        grid_row: GridPlacement::start(1),
                                        grid_column: GridPlacement::start(1),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(8),
                                        overflow: Overflow::scroll_y(),
                                        ..scroll_region::scroll_node()
                                    },
                                    BackgroundColor(Color::NONE),
                                    scroll_region::ScrollRegion,
                                    LeftPanelInputRegion,
                                    ScrollPosition::default(),
                                    Interaction::None,
                                    RelativeCursorPosition::default(),
                                ))
                                .with_children(|left_content| {
                                    left_content.spawn((Text::new(left_panel::TITLE),));
                                    left_content.spawn((Text::new(""),));

                                    spawn_section_header(
                                        left_content,
                                        "Sprite Sheet",
                                        LeftPanelSection::SpriteSheet,
                                    );
                                    left_content
                                        .spawn((
                                            Node {
                                                width: percent(100),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: px(6),
                                                padding: UiRect::left(px(6)),
                                                ..default()
                                            },
                                            SectionBody {
                                                section: LeftPanelSection::SpriteSheet,
                                            },
                                        ))
                                        .with_children(|sheet| {
                                            sheet
                                                .spawn((Text::new("Image: (none)"), ImageNameText));
                                            sheet.spawn((
                                                Text::new("Resolution: (none)"),
                                                ImageSizeText,
                                            ));
                                            sheet.spawn((Text::new("Status: idle"), StatusText));
                                        });

                                    spawn_section_header(
                                        left_content,
                                        "Grid Settings",
                                        LeftPanelSection::GridSettings,
                                    );
                                    left_content
                                        .spawn((
                                            Node {
                                                width: percent(100),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: px(6),
                                                padding: UiRect::left(px(6)),
                                                ..default()
                                            },
                                            SectionBody {
                                                section: LeftPanelSection::GridSettings,
                                            },
                                        ))
                                        .with_children(|grid_section| {
                                            spawn_numeric_field(
                                                grid_section,
                                                "Rows",
                                                GridFieldKind::Rows,
                                            );
                                            spawn_numeric_field(
                                                grid_section,
                                                "Columns",
                                                GridFieldKind::Columns,
                                            );
                                            spawn_numeric_field(
                                                grid_section,
                                                "Cell Width",
                                                GridFieldKind::CellWidth,
                                            );
                                            spawn_numeric_field(
                                                grid_section,
                                                "Cell Height",
                                                GridFieldKind::CellHeight,
                                            );
                                            spawn_numeric_field(
                                                grid_section,
                                                "Offset X",
                                                GridFieldKind::OffsetX,
                                            );
                                            spawn_numeric_field(
                                                grid_section,
                                                "Offset Y",
                                                GridFieldKind::OffsetY,
                                            );
                                            spawn_button(
                                                grid_section,
                                                "Clear Selection",
                                                ClearSelectionButton,
                                            );
                                        });

                                    spawn_section_header(
                                        left_content,
                                        "Parts",
                                        LeftPanelSection::Parts,
                                    );
                                    left_content
                                        .spawn((
                                            Node {
                                                width: percent(100),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: px(6),
                                                padding: UiRect::left(px(6)),
                                                ..default()
                                            },
                                            SectionBody {
                                                section: LeftPanelSection::Parts,
                                            },
                                        ))
                                        .with_children(|parts| {
                                            parts.spawn((
                                                Text::new("Parts: scanning catalog..."),
                                                TextFont::from_font_size(10.0),
                                                PartsStatusText,
                                            ));
                                            for layer in LayerCode::ALL {
                                                spawn_part_layer_row(parts, layer);
                                            }
                                        });

                                    spawn_section_header(
                                        left_content,
                                        "Palettes",
                                        LeftPanelSection::Palettes,
                                    );
                                    left_content
                                        .spawn((
                                            Node {
                                                width: percent(100),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: px(6),
                                                padding: UiRect::left(px(6)),
                                                ..default()
                                            },
                                            SectionBody {
                                                section: LeftPanelSection::Palettes,
                                            },
                                        ))
                                        .with_children(|palettes| {
                                            palettes.spawn((
                                                Text::new("Palettes: scanning catalog..."),
                                                TextFont::from_font_size(10.0),
                                                PaletteStatusText,
                                            ));
                                            palettes.spawn((
                                                Text::new("Current: (none)"),
                                                TextFont::from_font_size(10.0),
                                                PaletteCurrentText,
                                            ));
                                            palettes.spawn((
                                                Text::new("Path: -"),
                                                TextFont::from_font_size(10.0),
                                                PalettePathText,
                                            ));
                                            palettes
                                                .spawn(Node {
                                                    width: percent(100),
                                                    column_gap: px(6),
                                                    flex_wrap: FlexWrap::Wrap,
                                                    ..default()
                                                })
                                                .with_children(|row| {
                                                    spawn_button(
                                                        row,
                                                        "Prev Palette",
                                                        PaletteCycleButton { delta: -1 },
                                                    );
                                                    spawn_button(
                                                        row,
                                                        "Next Palette",
                                                        PaletteCycleButton { delta: 1 },
                                                    );
                                                });
                                        });

                                    spawn_section_header(
                                        left_content,
                                        "Animations",
                                        LeftPanelSection::Animations,
                                    );
                                    left_content
                                        .spawn((
                                            Node {
                                                width: percent(100),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: px(6),
                                                padding: UiRect::left(px(6)),
                                                ..default()
                                            },
                                            SectionBody {
                                                section: LeftPanelSection::Animations,
                                            },
                                        ))
                                        .with_children(|anim| {
                                            anim.spawn((
                                                Text::new("Animations > (none)"),
                                                ActiveAnimationText,
                                            ));
                                            anim.spawn((
                                                Text::new("Status: Viewing"),
                                                ActiveTrackText,
                                            ));
                                            anim.spawn((Text::new("Frames: 0"), PlaybackText));

                                            for (category, clip_indices) in
                                                grouped_clip_indices_by_category(
                                                    &anim_state.project.clips,
                                                )
                                            {
                                                spawn_animation_category_header(anim, &category);
                                                anim.spawn((
                                                    Node {
                                                        width: percent(100),
                                                        flex_direction: FlexDirection::Column,
                                                        row_gap: px(4),
                                                        padding: UiRect::left(px(10)),
                                                        ..default()
                                                    },
                                                    AnimationCategoryBody {
                                                        category: category.clone(),
                                                    },
                                                ))
                                                .with_children(|clip_list| {
                                                    for clip_index in clip_indices {
                                                        let label = anim_state.project.clips
                                                            [clip_index]
                                                            .display
                                                            .clone();
                                                        spawn_button(
                                                            clip_list,
                                                            &label,
                                                            AnimationClipButton { clip_index },
                                                        );
                                                    }
                                                });
                                            }

                                            anim.spawn((Text::new("Direction"),));
                                            anim.spawn((Node {
                                                width: percent(100),
                                                column_gap: px(6),
                                                flex_wrap: FlexWrap::Wrap,
                                                ..default()
                                            },))
                                                .with_children(|row| {
                                                    for direction in Direction::ALL {
                                                        spawn_direction_button(row, direction);
                                                    }
                                                });

                                            anim.spawn((Text::new("Frames Preview"),));
                                            anim.spawn((
                                                Node {
                                                    display: Display::Grid,
                                                    width: percent(100),
                                                    min_height: px(98),
                                                    border: UiRect::all(px(1)),
                                                    grid_template_columns: vec![
                                                        RepeatedGridTrack::flex(1, 1.0),
                                                    ],
                                                    grid_template_rows: vec![
                                                        RepeatedGridTrack::auto(1),
                                                        RepeatedGridTrack::auto(1),
                                                    ],
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgb(0.09, 0.09, 0.12)),
                                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                                            ))
                                            .with_children(|preview_frame| {
                                                let scroll_id = preview_frame
                                                    .spawn((
                                                        Node {
                                                            grid_row: GridPlacement::start(1),
                                                            grid_column: GridPlacement::start(1),
                                                            width: percent(100),
                                                            min_height: px(88),
                                                            padding: UiRect::all(px(6)),
                                                            overflow: Overflow::scroll_x(),
                                                            ..default()
                                                        },
                                                        ScrollPosition::default(),
                                                        Interaction::None,
                                                        RelativeCursorPosition::default(),
                                                    ))
                                                    .with_children(|preview| {
                                                        preview.spawn((
                                                            Node {
                                                                width: auto(),
                                                                flex_direction: FlexDirection::Row,
                                                                column_gap: px(6),
                                                                ..default()
                                                            },
                                                            PreviewStripContent,
                                                        ));
                                                    })
                                                    .id();
                                                spawn_horizontal_scrollbar(
                                                    preview_frame,
                                                    scroll_id,
                                                );
                                            });

                                            anim.spawn((Text::new("Metadata"),));
                                            anim.spawn((Node {
                                                width: percent(100),
                                                column_gap: px(6),
                                                flex_wrap: FlexWrap::Wrap,
                                                ..default()
                                            },))
                                                .with_children(|row| {
                                                    spawn_button(
                                                        row,
                                                        "Append Mirrored Copy",
                                                        AppendMirroredCopyButton,
                                                    );
                                                    spawn_button(
                                                        row,
                                                        "Flip Horizontally",
                                                        ToggleStepFlipButton,
                                                    );
                                                });

                                            spawn_animation_text_field(
                                                anim,
                                                "Duration (ms)",
                                                AnimationFieldKind::StepMs,
                                            );
                                            anim.spawn((Node {
                                                width: percent(100),
                                                column_gap: px(6),
                                                ..default()
                                            },))
                                                .with_children(|row| {
                                                    spawn_button(
                                                        row,
                                                        "Apply Duration",
                                                        ApplyStepMsButton,
                                                    );
                                                });

                                            anim.spawn((Text::new("Playback"),));
                                            anim.spawn((Node {
                                                width: percent(100),
                                                column_gap: px(6),
                                                flex_wrap: FlexWrap::Wrap,
                                                ..default()
                                            },))
                                                .with_children(|row| {
                                                    spawn_button(
                                                        row,
                                                        "Loop",
                                                        SetPlaybackLoopButton,
                                                    );
                                                    spawn_button(
                                                        row,
                                                        "One Shot",
                                                        SetPlaybackOneShotButton,
                                                    );
                                                    spawn_button(
                                                        row,
                                                        "Loop N",
                                                        SetPlaybackLoopNButton,
                                                    );
                                                });
                                            spawn_animation_text_field(
                                                anim,
                                                "Loop N Times",
                                                AnimationFieldKind::LoopNTimes,
                                            );
                                            anim.spawn((Node {
                                                width: percent(100),
                                                column_gap: px(6),
                                                ..default()
                                            },))
                                                .with_children(|row| {
                                                    spawn_button(
                                                        row,
                                                        "Apply Loop N",
                                                        ApplyLoopNTimesButton,
                                                    );
                                                });
                                            spawn_animation_text_field(
                                                anim,
                                                "Hold Last Frame",
                                                AnimationFieldKind::HoldLast,
                                            );
                                            anim.spawn((Node {
                                                width: percent(100),
                                                column_gap: px(6),
                                                ..default()
                                            },))
                                                .with_children(|row| {
                                                    spawn_button(
                                                        row,
                                                        "Apply Hold Last",
                                                        ToggleHoldLastButton,
                                                    );
                                                });
                                        });
                                })
                                .id();

                            spawn_vertical_scrollbar(left_frame, scroll_id);
                        });
                    });

                main_area.spawn((
                    Button,
                    Node {
                        width: px(splitters::SPLITTER_SIZE),
                        height: percent(100),
                        ..default()
                    },
                    splitters::splitter_color(),
                    splitters::VerticalSplitter,
                    SplitterHandle {
                        kind: SplitterKind::Left,
                    },
                ));

                main_area
                    .spawn((
                        Name::new("CenterWorkspace"),
                        Node {
                            flex_grow: 1.0,
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ))
                    .with_children(|center_workspace| {
                        center_workspace
                            .spawn((
                                Name::new("GridCanvas"),
                                Node {
                                    flex_grow: 1.0,
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(px(12)),
                                    ..default()
                                },
                                center_canvas::background(),
                            ))
                            .with_children(|center| {
                                center.spawn((Text::new("Selected Cells: 0"), SelectionCountText));
                                center
                                    .spawn(Node {
                                        display: Display::Grid,
                                        width: percent(100),
                                        flex_grow: 1.0,
                                        grid_template_columns: vec![
                                            RepeatedGridTrack::flex(1, 1.0),
                                            RepeatedGridTrack::auto(1),
                                        ],
                                        ..default()
                                    })
                                    .with_children(|viewport_frame| {
                                        viewport_frame
                                            .spawn((
                                                Node {
                                                    grid_row: GridPlacement::start(1),
                                                    grid_column: GridPlacement::start(1),
                                                    width: percent(100),
                                                    flex_grow: 1.0,
                                                    padding: UiRect::all(px(8)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.15)),
                                            ))
                                            .with_children(|viewport_outer| {
                                                viewport_outer
                                                    .spawn((
                                                        Node {
                                                            width: percent(100),
                                                            flex_grow: 1.0,
                                                            overflow: Overflow::clip(),
                                                            ..default()
                                                        },
                                                        Interaction::None,
                                                        RelativeCursorPosition::default(),
                                                        CanvasViewport,
                                                    ))
                                                    .with_children(|viewport| {
                                                        viewport.spawn((
                                                            Node {
                                                                width: px((grid.columns
                                                                    * grid.cell_width)
                                                                    as f32),
                                                                height: px((grid.rows
                                                                    * grid.cell_height)
                                                                    as f32),
                                                                position_type:
                                                                    PositionType::Absolute,
                                                                ..default()
                                                            },
                                                            CanvasSurface,
                                                        ));
                                                    });
                                            });
                                    });
                            });

                        center_workspace.spawn((
                            Button,
                            Node {
                                width: percent(100),
                                height: px(splitters::SPLITTER_SIZE),
                                ..default()
                            },
                            splitters::splitter_color(),
                            splitters::HorizontalSplitter,
                            SplitterHandle {
                                kind: SplitterKind::Bottom,
                            },
                        ));

                        center_workspace
                            .spawn((
                                Name::new("BottomMiddlePanel"),
                                bottom_panel::node(layout.bottom_panel_height),
                                center_canvas::background(),
                                BottomPanelHeightNode,
                            ))
                            .with_children(|bottom| {
                                bottom
                                    .spawn(Node {
                                        display: Display::Grid,
                                        width: percent(100),
                                        height: percent(100),
                                        grid_template_columns: vec![
                                            RepeatedGridTrack::flex(1, 1.0),
                                            RepeatedGridTrack::auto(1),
                                        ],
                                        ..default()
                                    })
                                    .with_children(|bottom_frame| {
                                        let scroll_id = bottom_frame
                                            .spawn((
                                                Node {
                                                    grid_row: GridPlacement::start(1),
                                                    grid_column: GridPlacement::start(1),
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: px(8),
                                                    overflow: Overflow::scroll_y(),
                                                    ..scroll_region::scroll_node()
                                                },
                                                BackgroundColor(Color::NONE),
                                                scroll_region::ScrollRegion,
                                                BottomPanelInputRegion,
                                                ScrollPosition::default(),
                                                Interaction::None,
                                                RelativeCursorPosition::default(),
                                            ))
                                            .with_children(|bottom_content| {
                                                bottom_content.spawn((Text::new(bottom_panel::TITLE),));
                                                bottom_content.spawn((
                                                    Text::new("Timeline tools are staged for a later pass."),
                                                    TextFont::from_font_size(11.0),
                                                ));
                                            })
                                            .id();

                                        spawn_vertical_scrollbar(bottom_frame, scroll_id);
                                    });
                            });
                    });

                main_area.spawn((
                    Button,
                    Node {
                        width: px(splitters::SPLITTER_SIZE),
                        height: percent(100),
                        ..default()
                    },
                    splitters::splitter_color(),
                    splitters::VerticalSplitter,
                    SplitterHandle {
                        kind: SplitterKind::Right,
                    },
                ));

                main_area
                    .spawn((
                        Name::new("RightPanel"),
                        right_panel::node(layout.right_panel_width),
                        right_panel::background(),
                        RightPanelWidthNode,
                    ))
                    .with_children(|right| {
                        right
                            .spawn(Node {
                                width: percent(100),
                                height: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(8),
                                ..default()
                            })
                            .with_children(|right_content| {
                                right_content.spawn((Text::new("Animation Viewer"),));
                                right_content
                                    .spawn((
                                        Node {
                                            width: percent(100),
                                            min_height: px(280),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(px(1)),
                                            position_type: PositionType::Relative,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.08, 0.08, 0.10)),
                                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                                    ))
                                    .with_children(|viewer| {
                                        viewer
                                            .spawn(Node {
                                                width: px(220),
                                                height: px(220),
                                                position_type: PositionType::Relative,
                                                ..default()
                                            })
                                            .with_children(|stack| {
                                                for layer in LayerCode::ALL {
                                                    stack.spawn((
                                                        ImageNode::default(),
                                                        Node {
                                                            width: px(220),
                                                            height: px(220),
                                                            position_type: PositionType::Absolute,
                                                            left: px(0),
                                                            top: px(0),
                                                            display: if layer == LayerCode::Body01 {
                                                                Display::Flex
                                                            } else {
                                                                Display::None
                                                            },
                                                            ..default()
                                                        },
                                                        ViewerLayerImageNode { layer },
                                                    ));
                                                }
                                            });
                                        viewer
                                            .spawn((
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    top: px(6),
                                                    left: px(6),
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: px(2),
                                                    padding: UiRect::all(px(4)),
                                                    border: UiRect::all(px(1)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.80)),
                                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                                            ))
                                            .with_children(|overlay| {
                                                overlay.spawn((
                                                    Text::new("Clip: (none)"),
                                                    TextFont::from_font_size(10.0),
                                                    ViewerClipText,
                                                ));
                                                overlay.spawn((
                                                    Text::new("Cell: -"),
                                                    TextFont::from_font_size(10.0),
                                                    ViewerCellText,
                                                ));
                                                overlay.spawn((
                                                    Text::new("Step: 0/0"),
                                                    TextFont::from_font_size(10.0),
                                                    ViewerStepText,
                                                ));
                                                overlay.spawn((
                                                    Text::new("Ms: -"),
                                                    TextFont::from_font_size(10.0),
                                                    ViewerMsText,
                                                ));
                                                overlay.spawn((
                                                    Text::new("Flip: -"),
                                                    TextFont::from_font_size(10.0),
                                                    ViewerFlipText,
                                                ));
                                            });
                                    });

                                right_content
                                    .spawn(Node {
                                        display: Display::Grid,
                                        width: percent(100),
                                        flex_grow: 1.0,
                                        min_height: px(0),
                                        grid_template_columns: vec![
                                            RepeatedGridTrack::flex(1, 1.0),
                                            RepeatedGridTrack::auto(1),
                                        ],
                                        ..default()
                                    })
                                    .with_children(|right_frame| {
                                        let scroll_id = right_frame
                                            .spawn((
                                                Node {
                                                    grid_row: GridPlacement::start(1),
                                                    grid_column: GridPlacement::start(1),
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: px(8),
                                                    min_height: px(0),
                                                    overflow: Overflow::scroll_y(),
                                                    ..scroll_region::scroll_node()
                                                },
                                                BackgroundColor(Color::NONE),
                                                scroll_region::ScrollRegion,
                                                RightPanelInputRegion,
                                                ScrollPosition::default(),
                                                Interaction::None,
                                                RelativeCursorPosition::default(),
                                            ))
                                            .with_children(|scroll_content| {
                                                spawn_right_panel_section_header(
                                                    scroll_content,
                                                    "Playback",
                                                );
                                                scroll_content
                                                    .spawn((
                                                        Node {
                                                            width: percent(100),
                                                            flex_direction: FlexDirection::Column,
                                                            row_gap: px(8),
                                                            padding: UiRect::left(px(6)),
                                                            ..default()
                                                        },
                                                        RightPanelPlaybackSectionBody,
                                                    ))
                                                    .with_children(|playback| {
                                                        playback.spawn((Text::new("Direction"),));
                                                        playback
                                                            .spawn((Node {
                                                                width: percent(100),
                                                                column_gap: px(6),
                                                                flex_wrap: FlexWrap::Wrap,
                                                                ..default()
                                                            },))
                                                            .with_children(|row| {
                                                                for direction in Direction::ALL {
                                                                    spawn_viewer_direction_button(
                                                                        row, direction,
                                                                    );
                                                                }
                                                            });

                                                        playback.spawn((Text::new("Transport"),));
                                                        playback
                                                            .spawn((Node {
                                                                width: percent(100),
                                                                column_gap: px(6),
                                                                flex_wrap: FlexWrap::Wrap,
                                                                ..default()
                                                            },))
                                                            .with_children(|row| {
                                                                row.spawn((
                                                                    Button,
                                                                    Node {
                                                                        padding: UiRect::axes(
                                                                            px(8),
                                                                            px(6),
                                                                        ),
                                                                        border: UiRect::all(px(1)),
                                                                        ..default()
                                                                    },
                                                                    BackgroundColor(Color::srgb(
                                                                        0.16, 0.16, 0.20,
                                                                    )),
                                                                    BorderColor::all(Color::srgba(
                                                                        1.0, 1.0, 1.0, 0.20,
                                                                    )),
                                                                    ViewerPlayPauseButton,
                                                                ))
                                                                .with_children(|button| {
                                                                    button.spawn((
                                                                        Text::new("Pause"),
                                                                        ViewerPlayPauseLabel,
                                                                    ));
                                                                });
                                                                spawn_button(
                                                                    row,
                                                                    "Prev",
                                                                    ViewerPrevFrameButton,
                                                                );
                                                                spawn_button(
                                                                    row,
                                                                    "Next",
                                                                    ViewerNextFrameButton,
                                                                );
                                                            });

                                                        playback.spawn((Text::new("Speed"),));
                                                        playback
                                                            .spawn((Node {
                                                                width: percent(100),
                                                                column_gap: px(6),
                                                                flex_wrap: FlexWrap::Wrap,
                                                                ..default()
                                                            },))
                                                            .with_children(|row| {
                                                                spawn_viewer_speed_button(
                                                                    row, "0.5x", 0.5,
                                                                );
                                                                spawn_viewer_speed_button(
                                                                    row, "1x", 1.0,
                                                                );
                                                                spawn_viewer_speed_button(
                                                                    row, "2x", 2.0,
                                                                );
                                                            });

                                                        playback.spawn((Text::new("Loop Override"),));
                                                        playback
                                                            .spawn((
                                                                Button,
                                                                Node {
                                                                    padding: UiRect::axes(px(8), px(6)),
                                                                    border: UiRect::all(px(1)),
                                                                    ..default()
                                                                },
                                                                BackgroundColor(Color::srgb(
                                                                    0.16, 0.16, 0.20,
                                                                )),
                                                                BorderColor::all(Color::srgba(
                                                                    1.0, 1.0, 1.0, 0.20,
                                                                )),
                                                                ViewerLoopOverrideButton,
                                                            ))
                                                            .with_children(|button| {
                                                                button.spawn((
                                                                    Text::new("Off"),
                                                                    ViewerLoopOverrideLabel,
                                                                ));
                                                            });
                                                    });

                                                spawn_right_panel_outfit_section_header(
                                                    scroll_content,
                                                    "Outfits",
                                                );
                                                scroll_content
                                                    .spawn((
                                                        Node {
                                                            width: percent(100),
                                                            flex_direction: FlexDirection::Column,
                                                            row_gap: px(8),
                                                            padding: UiRect::left(px(6)),
                                                            ..default()
                                                        },
                                                        RightPanelOutfitSectionBody,
                                                    ))
                                                    .with_children(|outfits| {
                                                        outfits
                                                            .spawn(Node {
                                                                width: percent(100),
                                                                column_gap: px(6),
                                                                flex_wrap: FlexWrap::Wrap,
                                                                ..default()
                                                            })
                                                            .with_children(|row| {
                                                                spawn_button(
                                                                    row,
                                                                    "Add Outfit",
                                                                    AddOutfitButton,
                                                                );
                                                                spawn_button(
                                                                    row,
                                                                    "Save Changes",
                                                                    SaveOutfitChangesButton,
                                                                );
                                                                spawn_button(
                                                                    row,
                                                                    "Delete Outfit",
                                                                    DeleteOutfitButton,
                                                                );
                                                            });

                                                        outfits.spawn((Text::new("Filter"),));
                                                        spawn_outfit_filter_text_field(
                                                            outfits,
                                                            "Search",
                                                        );
                                                        outfits
                                                            .spawn(Node {
                                                                width: percent(100),
                                                                column_gap: px(6),
                                                                flex_wrap: FlexWrap::Wrap,
                                                                ..default()
                                                            })
                                                            .with_children(|row| {
                                                                spawn_button(
                                                                    row,
                                                                    "Add Filter Tag",
                                                                    AddOutfitFilterTagButton,
                                                                );
                                                                spawn_button(
                                                                    row,
                                                                    "Clear Filters",
                                                                    ClearOutfitFiltersButton,
                                                                );
                                                            });
                                                        outfits
                                                            .spawn((
                                                                Node {
                                                                    width: percent(100),
                                                                    column_gap: px(6),
                                                                    row_gap: px(4),
                                                                    flex_wrap: FlexWrap::Wrap,
                                                                    ..default()
                                                                },
                                                                OutfitFilterChipsContainer,
                                                            ));
                                                        outfits
                                                            .spawn((
                                                                Node {
                                                                    width: percent(100),
                                                                    column_gap: px(6),
                                                                    row_gap: px(4),
                                                                    flex_wrap: FlexWrap::Wrap,
                                                                    ..default()
                                                                },
                                                                OutfitFilterAutocompleteContainer,
                                                            ));

                                                        outfits.spawn((Text::new("Outfit List"),));
                                                        outfits
                                                            .spawn((
                                                                Node {
                                                                    width: percent(100),
                                                                    max_height: px(180),
                                                                    min_height: px(120),
                                                                    border: UiRect::all(px(1)),
                                                                    padding: UiRect::all(px(4)),
                                                                    flex_direction:
                                                                        FlexDirection::Column,
                                                                    row_gap: px(4),
                                                                    overflow: Overflow::scroll_y(),
                                                                    ..default()
                                                                },
                                                                BackgroundColor(Color::srgb(
                                                                    0.10, 0.10, 0.13,
                                                                )),
                                                                BorderColor::all(Color::srgba(
                                                                    1.0, 1.0, 1.0, 0.20,
                                                                )),
                                                                ScrollPosition::default(),
                                                            ))
                                                            .with_children(|list| {
                                                                list.spawn((
                                                                    Node {
                                                                        width: percent(100),
                                                                        flex_direction:
                                                                            FlexDirection::Column,
                                                                        row_gap: px(4),
                                                                        ..default()
                                                                    },
                                                                    OutfitListContainer,
                                                                ));
                                                            });

                                                        outfits.spawn((
                                                            Text::new("Selected: (none)"),
                                                            OutfitIdentityText,
                                                        ));
                                                        spawn_outfit_text_field(
                                                            outfits,
                                                            "Outfit ID",
                                                            OutfitFieldKind::OutfitId,
                                                        );
                                                        spawn_outfit_text_field(
                                                            outfits,
                                                            "Display Name",
                                                            OutfitFieldKind::DisplayName,
                                                        );
                                                        spawn_outfit_text_field(
                                                            outfits,
                                                            "Tag Input",
                                                            OutfitFieldKind::TagInput,
                                                        );
                                                        outfits
                                                            .spawn(Node {
                                                                width: percent(100),
                                                                column_gap: px(6),
                                                                flex_wrap: FlexWrap::Wrap,
                                                                ..default()
                                                            })
                                                            .with_children(|row| {
                                                                spawn_button(
                                                                    row,
                                                                    "Add Tag",
                                                                    AddOutfitTagButton,
                                                                );
                                                            });
                                                        outfits
                                                            .spawn((
                                                                Node {
                                                                    width: percent(100),
                                                                    column_gap: px(6),
                                                                    row_gap: px(4),
                                                                    flex_wrap: FlexWrap::Wrap,
                                                                    ..default()
                                                                },
                                                                OutfitTagChipsContainer,
                                                            ));

                                                        outfits.spawn((Text::new("Summary"),));
                                                        outfits.spawn((
                                                            Text::new("Equipped parts (preview):\n(none)"),
                                                            TextFont::from_font_size(10.0),
                                                            OutfitSummaryText,
                                                        ));
                                                        outfits.spawn((
                                                            Text::new("Outfits: 0"),
                                                            TextFont::from_font_size(10.0),
                                                            OutfitStatusText,
                                                        ));
                                                    });
                                            })
                                            .id();

                                        spawn_vertical_scrollbar(right_frame, scroll_id);
                                    });
                            });
                    });
            });
        });
}

pub(super) fn spawn_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: T,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(8), px(6)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            marker,
        ))
        .with_children(|button| {
            button.spawn((Text::new(label),));
        });
}

pub(super) fn spawn_direction_button(parent: &mut ChildSpawnerCommands, direction: Direction) {
    let label = if direction == Direction::Left {
        "Left*"
    } else {
        direction.label()
    };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(8), px(6)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            DirectionButton { direction },
            DirectionButtonStyleMarker { direction },
        ))
        .with_children(|button| {
            button.spawn((Text::new(label),));
        });
}

pub(super) fn spawn_viewer_direction_button(
    parent: &mut ChildSpawnerCommands,
    direction: Direction,
) {
    let label = if direction == Direction::Left {
        "Left*"
    } else {
        direction.label()
    };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(8), px(6)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            ViewerDirectionButton { direction },
            ViewerDirectionButtonStyleMarker { direction },
        ))
        .with_children(|button| {
            button.spawn((Text::new(label),));
        });
}

pub(super) fn spawn_viewer_speed_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    speed: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(8), px(6)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            ViewerSpeedButton { speed },
            ViewerSpeedButtonStyleMarker { speed },
        ))
        .with_children(|button| {
            button.spawn((Text::new(label),));
        });
}

pub(super) fn spawn_part_layer_row(parent: &mut ChildSpawnerCommands, layer: LayerCode) {
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        })
        .with_children(|container| {
            container
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(layer.as_str()),
                        TextFont::from_font_size(10.0),
                        Node {
                            min_width: px(56),
                            ..default()
                        },
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: px(22),
                            height: px(22),
                            border: UiRect::all(px(1)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                        PartCycleButton { layer, delta: -1 },
                    ))
                    .with_children(|button| {
                        button.spawn((Text::new("<"), TextFont::from_font_size(10.0)));
                    });
                    row.spawn((
                        Text::new("(none)"),
                        TextFont::from_font_size(10.0),
                        Node {
                            min_width: px(126),
                            ..default()
                        },
                        PartCurrentText { layer },
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: px(22),
                            height: px(22),
                            border: UiRect::all(px(1)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                        PartCycleButton { layer, delta: 1 },
                    ))
                    .with_children(|button| {
                        button.spawn((Text::new(">"), TextFont::from_font_size(10.0)));
                    });
                });

            container
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                })
                .with_children(|palette_row| {
                    palette_row.spawn((
                        Text::new("color"),
                        TextFont::from_font_size(9.0),
                        Node {
                            min_width: px(56),
                            ..default()
                        },
                    ));
                    palette_row
                        .spawn((
                            Button,
                            Node {
                                width: px(22),
                                height: px(20),
                                border: UiRect::all(px(1)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                            PartPaletteCycleButton { layer, delta: -1 },
                        ))
                        .with_children(|button| {
                            button.spawn((Text::new("<"), TextFont::from_font_size(10.0)));
                        });
                    palette_row.spawn((
                        Text::new("(global)"),
                        TextFont::from_font_size(9.0),
                        Node {
                            min_width: px(126),
                            ..default()
                        },
                        PartPaletteCurrentText { layer },
                    ));
                    palette_row
                        .spawn((
                            Button,
                            Node {
                                width: px(22),
                                height: px(20),
                                border: UiRect::all(px(1)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                            PartPaletteCycleButton { layer, delta: 1 },
                        ))
                        .with_children(|button| {
                            button.spawn((Text::new(">"), TextFont::from_font_size(10.0)));
                        });
                });
        });
}

pub(super) fn spawn_animation_category_header(parent: &mut ChildSpawnerCommands, category: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(6), px(4)),
                column_gap: px(6),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            AnimationCategoryButton {
                category: category.to_string(),
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(""),
                AnimationCategoryArrowText {
                    category: category.to_string(),
                },
            ));
            row.spawn((Text::new(category),));
        });
}

pub(super) fn grouped_clip_indices_by_category(
    clips: &[crate::features::animation::Clip],
) -> Vec<(String, Vec<usize>)> {
    let mut categories: Vec<(String, Vec<usize>)> = Vec::new();
    for (clip_index, clip) in clips.iter().enumerate() {
        if let Some((_, indices)) = categories
            .iter_mut()
            .find(|(category, _)| *category == clip.category)
        {
            indices.push(clip_index);
        } else {
            categories.push((clip.category.clone(), vec![clip_index]));
        }
    }
    categories
}

pub(super) fn format_part_short(part: &crate::features::layers::PartDef) -> String {
    let mut label = format!("{}_{:02}", part.part_id.name, part.part_id.version);
    if let Some(palette) = part.part_id.palette {
        label.push(palette);
    }
    if part.part_id.special.is_some() {
        label.push_str("_e");
    }
    label
}

pub(super) fn spawn_section_header(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    section: LeftPanelSection,
) {
    parent
        .spawn((
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(6), px(4)),
                column_gap: px(6),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.13, 0.13, 0.16)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            SectionToggleButton { section },
        ))
        .with_children(|row| {
            row.spawn((Text::new(""), SectionToggleText { section }));
            row.spawn((Text::new(title),));
        });
}

pub(super) fn spawn_right_panel_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(6), px(4)),
                column_gap: px(6),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.13, 0.13, 0.16)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            RightPanelPlaybackSectionToggleButton,
        ))
        .with_children(|row| {
            row.spawn((Text::new("v"), RightPanelPlaybackSectionToggleText));
            row.spawn((Text::new(title),));
        });
}

pub(super) fn spawn_right_panel_outfit_section_header(
    parent: &mut ChildSpawnerCommands,
    title: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(6), px(4)),
                column_gap: px(6),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.13, 0.13, 0.16)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            RightPanelOutfitSectionToggleButton,
        ))
        .with_children(|row| {
            row.spawn((Text::new("v"), RightPanelOutfitSectionToggleText));
            row.spawn((Text::new(title),));
        });
}

pub(super) fn spawn_outfit_text_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    field: OutfitFieldKind,
) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                Node {
                    min_width: px(110),
                    ..default()
                },
            ));
            row.spawn((
                Button,
                Node {
                    width: px(146),
                    padding: UiRect::axes(px(8), px(5)),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.10, 0.13)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                OutfitFieldButton { field },
            ))
            .with_children(|field_node| {
                field_node.spawn((Text::new(""), OutfitFieldText { field }));
            });
        });
}

pub(super) fn spawn_outfit_filter_text_field(parent: &mut ChildSpawnerCommands, label: &str) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                Node {
                    min_width: px(110),
                    ..default()
                },
            ));
            row.spawn((
                Button,
                Node {
                    width: px(146),
                    padding: UiRect::axes(px(8), px(5)),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.10, 0.13)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                OutfitFilterFieldButton,
            ))
            .with_children(|field_node| {
                field_node.spawn((Text::new(""), OutfitFilterFieldText));
            });
        });
}

pub(super) fn spawn_numeric_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    field: GridFieldKind,
) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                Node {
                    min_width: px(90),
                    ..default()
                },
            ));
            row.spawn((
                Button,
                Node {
                    width: px(118),
                    padding: UiRect::axes(px(8), px(5)),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.10, 0.13)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                GridFieldButton { field },
            ))
            .with_children(|field_node| {
                field_node.spawn((Text::new(""), GridFieldText { field }));
            });
        });
}

pub(super) fn spawn_animation_text_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    field: AnimationFieldKind,
) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                Node {
                    min_width: px(110),
                    ..default()
                },
            ));
            row.spawn((
                Button,
                Node {
                    width: px(146),
                    padding: UiRect::axes(px(8), px(5)),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.10, 0.13)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                AnimationFieldButton { field },
            ))
            .with_children(|field_node| {
                field_node.spawn((Text::new(""), AnimationFieldText { field }));
            });
        });
}

pub(super) fn spawn_vertical_scrollbar(parent: &mut ChildSpawnerCommands, target: Entity) {
    parent
        .spawn((
            Node {
                min_width: px(10),
                grid_row: GridPlacement::start(1),
                grid_column: GridPlacement::start(2),
                ..default()
            },
            Scrollbar {
                orientation: ControlOrientation::Vertical,
                target,
                min_thumb_length: 16.0,
            },
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                Hovered::default(),
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                CoreScrollbarThumb,
            ));
        });
}

pub(super) fn spawn_horizontal_scrollbar(parent: &mut ChildSpawnerCommands, target: Entity) {
    parent
        .spawn((
            Node {
                min_height: px(10),
                grid_row: GridPlacement::start(2),
                grid_column: GridPlacement::start(1),
                ..default()
            },
            Scrollbar {
                orientation: ControlOrientation::Horizontal,
                target,
                min_thumb_length: 16.0,
            },
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                Hovered::default(),
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                CoreScrollbarThumb,
            ));
        });
}
