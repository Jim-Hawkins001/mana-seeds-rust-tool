# asset_hander - Step 1 Scaffold

This scaffold is created before implementation code for `STEP_1_PLAN.md`.

## Scope
Step 1 focuses on:
- image upload
- grid slicing + overlay rendering
- cell select / multi-select
- resizable + scrollable panel layout
- mode dropdown with only `Sprite Mode` active

Out-of-scope features (animation, collision, anchors, metadata persistence, export) are represented only as placeholders.

## Folder Structure

```text
asset_hander/
+- assets/
¦  +- images/
¦  +- fonts/
¦  +- icons/
+- doc/
¦  +- librarys guides/
¦     +- bevy_apis_guides/
+- src/
¦  +- main.rs
¦  +- app/
¦  ¦  +- mod.rs
¦  ¦  +- state.rs
¦  ¦  +- mode.rs
¦  +- ui/
¦  ¦  +- mod.rs
¦  ¦  +- shell.rs
¦  ¦  +- toolbar.rs
¦  ¦  +- panels/
¦  ¦  ¦  +- mod.rs
¦  ¦  ¦  +- left_panel.rs
¦  ¦  ¦  +- center_canvas.rs
¦  ¦  ¦  +- right_panel.rs
¦  ¦  ¦  +- bottom_panel.rs
¦  ¦  +- widgets/
¦  ¦     +- mod.rs
¦  ¦     +- splitters.rs
¦  ¦     +- scroll_region.rs
¦  +- features/
¦     +- mod.rs
¦     +- image/
¦     ¦  +- mod.rs
¦     ¦  +- loader.rs
¦     +- grid/
¦        +- mod.rs
¦        +- state.rs
¦        +- overlay.rs
¦        +- selection.rs
+- Cargo.toml
+- rust-toolchain.toml
```

## Ownership Map
- `src/app/*`: global state and mode wiring.
- `src/ui/shell.rs`: top/middle/bottom layout composition.
- `src/ui/toolbar.rs`: File/Edit/Mode bar and `Open Image` entry point.
- `src/ui/panels/left_panel.rs`: image + grid parameters.
- `src/ui/panels/center_canvas.rs`: image viewport + grid overlay + interactions.
- `src/ui/panels/right_panel.rs`: metadata placeholder container.
- `src/ui/panels/bottom_panel.rs`: animation player placeholder container.
- `src/ui/widgets/*`: splitters and scroll helpers for resizable/scrollable layout.
- `src/features/image/*`: image loading state and flow.
- `src/features/grid/*`: grid state, overlay visuals, and cell selection behavior.

## Notes
- Files currently contain scaffold placeholders only.
- Implementation starts after this structure lock.
