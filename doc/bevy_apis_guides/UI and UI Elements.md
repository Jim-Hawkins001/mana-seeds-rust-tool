UI and UI Elements (Bevy 0.18) - Latest Practical Guide

Latest status (confirmed):
- Bevy 0.18 is the latest release as of January 13, 2026.
- This guide targets Bevy 0.18 APIs and behavior.

1) UI Bootstrapping

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<bevy::input_focus::InputFocus>()
        .add_systems(Startup, setup_ui)
        .add_systems(Update, button_interaction_system)
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

Why:
- UI still needs a camera (`Camera2d` in typical 2D/top-down apps).
- `InputFocus` is required for proper focus/accessibility workflows used by modern Bevy UI examples.

2) Core UI Building Blocks

Primary components to use:
- `Node` (layout/style container)
- `Button` (interactive widget marker)
- `Text` (UI text)
- `ImageNode` (UI images)
- `BackgroundColor`, `BorderColor`, `BorderRadius`, `Outline`, `BoxShadow`

Layout model:
- Bevy UI uses Flexbox and CSS Grid style APIs via `Node`.
- Prefer composing many small `Node`s rather than one giant style node.

3) Layout Best Practices (Top-Down HUD + Menus)

Recommended patterns:
- Root fullscreen container: `width: percent(100), height: percent(100)`
- HUD bars: Flex row/column with explicit padding/margins
- Inventory/map grids: `Display::Grid` + `grid_template_*`
- Keep anchor layers explicit (top HUD, center overlays, modal layer)

Useful APIs:
- `Display::{Flex, Grid}`
- `UiRect`, `Val` helpers (`px()`, `percent()`)
- `ZIndex` and `GlobalZIndex` for layered UI ordering

4) Interaction Pattern That Scales

Use `Changed<Interaction>` queries for button-like controls:

```rust
fn button_interaction_system(
    mut q: Query<(&Interaction, &mut BackgroundColor), (With<Button>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in &mut q {
        *bg = match *interaction {
            Interaction::Pressed => Color::srgb(0.35, 0.75, 0.35).into(),
            Interaction::Hovered => Color::srgb(0.25, 0.25, 0.25).into(),
            Interaction::None => Color::srgb(0.15, 0.15, 0.15).into(),
        };
    }
}
```

Why:
- Avoids unnecessary per-frame UI mutation work.
- Scales better with many controls.

5) Keyboard/Gamepad Navigation (Important in 0.18)

New best path:
- Use automatic directional navigation for complex/dynamic menus.
- Add `AutoDirectionalNavigation` to focusable UI entities.
- Use the `AutoDirectionalNavigator` flow rather than manually wiring every edge.

When manual mapping still makes sense:
- Fixed, highly custom focus paths with non-spatial logic.

6) Scroll and Sticky Headers

New in 0.18:
- `IgnoreScroll` lets child UI ignore parent `ScrollPosition` axes.

Use cases:
- Sticky row/column headers in scrollable inventories or tables.
- Fixed mini-controls inside scrolling panels.

7) Text and Typography (0.18)

Current best usage:
- Use `TextFont` and set weight for variable fonts when needed.
- Use `Underline` and `Strikethrough` components directly where needed.
- Use `FontFeatures`/OpenType features for advanced typography in supported `.otf` fonts.
- Migration note: `line_height` is no longer on `TextFont`; use `LineHeight` component.

8) New Visual Styling Tools

Prefer native UI styling before custom shaders:
- `BackgroundGradient`, `BorderGradient`
- `BoxShadow`
- `BorderRadius`
- `Outline`

These are now mature enough for most game HUD/menu styling without custom render code.

9) Picking + Text Behavior Changes

0.18 behavior change:
- Non-text areas of `Text` nodes are no longer pickable.

If you want larger clickable text regions:
- Wrap text with an intermediate parent `Node`/`Button` and handle pointer/focus there.

10) Standard Widgets in 0.18

Bevy now includes more standard widget primitives (for example menu/popover-related pieces), but:
- Some widget APIs are still marked experimental (`bevy_ui_widgets` path).
- Use them for tooling/internal UI first, then lock usage once stable for shipping gameplay UI.

11) Performance Rules for UI-Heavy Games

- Prefer `Changed<T>` filters on interaction/style systems.
- Avoid rebuilding full UI trees every frame.
- Keep dynamic text updates localized (only update changed text entities).
- Use explicit layering (`ZIndex`/`GlobalZIndex`) to avoid hierarchy hacks.
- Keep expensive animations focused; use interpolation-aware APIs for `Val`/`Color` transitions where applicable.

12) Suggested Top-Down UI Architecture

Schedules:
- `Startup`: spawn root UI tree + persistent HUD
- `OnEnter(GameState::X)`: spawn state-specific screens
- `OnExit(GameState::X)`: despawn state-specific screens
- `Update`: interaction handling, text updates, focus/navigation logic

Data design:
- Marker components for widget roles (e.g., `HealthText`, `PauseMenuRoot`).
- Small UI systems per concern (navigation, style, text, sound cues).

13) Minimal Modern UI Example

```rust
use bevy::{input_focus::InputFocus, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputFocus>()
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_child((
            Button,
            Node {
                width: px(220),
                height: px(70),
                border: UiRect::all(px(3)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            children![Text::new("Start")],
        ));
}

fn button_system(
    mut q: Query<(&Interaction, &mut BackgroundColor), (With<Button>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in &mut q {
        bg.0 = match *interaction {
            Interaction::Pressed => Color::srgb(0.35, 0.75, 0.35),
            Interaction::Hovered => Color::srgb(0.25, 0.25, 0.25),
            Interaction::None => Color::srgb(0.15, 0.15, 0.15),
        };
    }
}
```

Sources (official)
- Bevy News: https://bevy.org/news/
- Bevy 0.18 release notes: https://bevy.org/news/bevy-0-18/
- 0.17 -> 0.18 migration guide: https://bevy.org/learn/migration-guides/0-17-to-0-18/
- Bevy UI crate docs (`latest`): https://docs.rs/bevy/latest/bevy/ui/
- Official UI examples index (button/text/scroll/etc): https://bevy.org/examples/
