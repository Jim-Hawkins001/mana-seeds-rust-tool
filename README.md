# Mana Seeds Rust Tool (`asset_hander`)

Sprite sheet, animation, paper-doll, and outfit authoring tool built with Bevy `0.18`.

## What It Does

- Loads sprite sheets from local `assets/`.
- Edits grid metadata (`rows`, `columns`, cell size, offsets).
- Authors animation clips and directional tracks (`Up`, `Down`, `Right`; `Left` derived/mirrored).
- Builds layered paper-doll previews from Mana Seed-style part sheets.
- Applies palette remaps from palette PNG ramps (including skin/hair ramp assets).
- Saves/loads animation projects as `.ron`.
- Manages outfit presets (`outfits.ron`) with equip + palette snapshots.

## Top-Level Modes

Use toolbar `Mode` to switch between:

- `Animations`
- `Parts`
- `Outfits`

Section defaults are mode-driven:

- `Animations`: animation authoring focus (left animations open, right playback open).
- `Parts`: parts/palettes focus (left parts + palettes open, right playback open).
- `Outfits`: preset authoring focus (left parts + palettes open, right outfits open).

## Right Panel

- Previewer stays visible at the top.
- `Playback` collapsible contains:
  - Direction
  - Transport (play/pause, prev/next)
  - Speed presets
  - Loop override
- `Outfits` collapsible contains:
  - `Add Outfit`, `Save Changes`, `Delete Outfit`
  - Filter field + `Add Filter Tag` + `Clear Filters`
  - Active filter chips (click chip to remove)
  - Tag autocomplete suggestions
  - Outfit list (filtered)
  - Editable identity fields (`Outfit ID`, `Display Name`, tag input)
  - Read-only preview summary (equipped parts + palette snapshot)

## Core Data Files

Recommended 3-file split:

1. Animation project (`*.ron`, commonly `humanoid_animations.ron`)
2. `parts_catalog.ron` (auto-generated from scanned parts)
3. `outfits.ron` (preset database)

### `parts_catalog.ron`

Generated from parsed part filenames and includes per-part metadata such as:

- stable `part_key`
- `layer`, `name`, `version`, optional palette letter, `_e` flag
- sprite `image_path`
- mapping rule (`FollowBodyCells`)
- slot conflict group
- paired-layer requirements

### `outfits.ron`

Stores outfit presets with:

- `outfit_id` (stable key)
- `display_name`
- `tags`
- `equipped` parts (`layer` + `part_key`)
- `palette` selection:
  - `skin`
  - `hair`
  - `outfit_main`
  - `outfit_accent` (optional)

Outfit save behavior follows "save what you see": equipped layers + current palette state are snapshotted from preview state.

## Current Feature Set

Implemented:

- File menu actions:
  - Open image
  - Load animation project
  - Save animation project
- Canvas and grid interaction:
  - Cell toggle editing for active tracks
  - Selection sync between workspace and preview strip
  - Panning and zooming
- Animation tools:
  - Category/clip tree
  - Direction arming/disarming
  - Step selection, append mirrored copy, flip toggles
  - Duration and playback field editing
- Viewer tools:
  - Direction picker
  - Play/pause
  - Prev/next frame stepping
  - Speed presets
  - Loop override
- Paper-doll tools:
  - Part catalog scanning from filenames
  - Equip/unequip cycling per layer (`none` state supported)
  - Slot conflict rules and paired cloak behavior
  - Hair/hat exclusion handling for `_e` variants
- Palette tools:
  - Global palette and variant cycling
  - Per-layer palette override cycling
  - Runtime remap table extraction from palette images
- Outfit tools:
  - Add/select/save/delete presets
  - Tag add/remove
  - Filter chips and autocomplete
  - Load preset -> applies equipped parts + palettes immediately
  - Save/load `outfits.ron`
- Project persistence:
  - Grid/sheet metadata
  - Animation clips, tracks, step timing, playback
  - Equipped parts and mapping metadata
  - Backward-compatible load for legacy `.ron` without `layers`

Not implemented yet:

- Collision editor
- Anchor editor
- Per-part custom animation override authoring UI

## Requirements

- Rust stable toolchain
- Cargo
- Windows is the currently tested environment

## Run

```powershell
cd C:\Users\yup\rust_project\asset_hander
cargo run
```

Main window title: `Mana Seeds Rust Tool`.

## Quick Workflow

1. Place sprites and palettes under `asset_hander/assets/...`.
2. Open a sprite sheet via `File -> Open Image`.
3. Set grid values in the left panel.
4. Switch modes with `Mode -> Animations / Parts / Outfits`.
5. In `Animations`, author clips/tracks and playback behavior.
6. In `Parts`, cycle layers and palette variants (`none` when cycling past range).
7. In `Outfits`, snapshot current preview state with `Add Outfit`, then edit ID/name/tags.
8. Use outfit filter chips/autocomplete to find presets quickly.
9. Save animation project with `File -> Save Anim Project`.
10. Save outfits with `Outfits -> Save Changes`.

## Input Notes

- Canvas zoom: mouse wheel over canvas.
- Panel scrolling: mouse wheel over left/right/bottom panels.
- Horizontal panel scroll: hold `Shift` while using wheel.
- Canvas pan: middle mouse drag, or `Space + Left Mouse` drag.

## Asset Naming Rules

Part sheets are parsed with:

`fbas_<layer>_<name>_<ver+palette>[_e].png`

Example:

`fbas_14head_headscarf_00b_e.png`

Constraints:

- Prefix must be `fbas`
- Layer code must be one of `00undr..15over`
- Version starts with two digits
- Optional palette suffix for version `00`: `a|b|c|d|f`
- Optional special suffix currently supported: `_e`

## Palette Ramp Expectations

Palette remaps are sampled from PNG ramps in assets (no hardcoded ramp tables).

Common ramp files:

- `palettes/mana seed 3-color ramps.png`
- `palettes/mana seed 4-color ramps.png`
- `palettes/mana seed skin ramps.png`
- `palettes/mana seed hair ramps.png`
- `palettes/base ramps/*.png`

## Development Commands

```powershell
cargo fmt
cargo check
cargo test
cargo clippy --all-targets -- -W clippy::all
```

## Code Layout

- `src/main.rs`: app bootstrap
- `src/app/*`: top-level app state and plugin wiring
- `src/features/image/*`: image loading state and sync
- `src/features/grid/*`: grid math and selection utilities
- `src/features/animation/mod.rs`: animation model and serialization
- `src/features/layers/mod.rs`: part/palette catalog and equip logic
- `src/features/outfits/mod.rs`: outfit db model + `outfits.ron` load/save
- `src/ui/shell/*`: shell setup, interaction systems, toolbar handlers, sync systems
- `src/ui/panels/*`: panel style and layout helpers
- `src/ui/widgets/*`: splitters and scroll helpers
