# asset_hander

Sprite sheet, animation, and paper-doll layer authoring tool built with `bevy = 0.18`.

## Current Status

Implemented:
- Image loading (`File -> Open Image`)
- Grid setup (`Rows`, `Columns`, `Cell Width`, `Cell Height`, `Offset X/Y`)
- Center workspace grid cell interaction
- Animation catalog/tree by category and clip
- Direction editing model (`Up`, `Down`, `Right` authored, `Left` derived/mirrored)
- Track editing by clicking workspace cells:
  - add new step when cell is not present
  - remove last occurrence when cell is already present
- Horizontal preview strip using atlas indices (`TextureAtlasLayout + TextureAtlas`)
- Step selection + metadata editing (`Duration (ms)`, flip)
- Clip playback mode authoring (`Loop`, `LoopN`, `OneShot + hold`)
- Right-panel animation viewer:
  - direction selector
  - play/pause, prev/next frame
  - speed presets
  - loop override
- Paper-doll layer catalog scan from assets (`fbas_<layer>_<name>_<ver+palette>[_e].png`)
- Palette ramp catalog scan from assets (`*/palettes/*.png`)
- Layer equip rules:
  - slot conflicts (`Lower`, `Footwear`, `Head`)
  - paired cloak set handling (`00undr + 11neck`)
  - `_e` hair/hat exclusion rule
- Layered viewer composition (body + equipped layers in `00undr..15over` order)
- Left-panel `Parts` section with per-layer cycle controls
- Left-panel `Palettes` section with palette browse controls
- Viewer color remapping from selected palette ramp + variant
- Per-layer color override controls in `Parts` (`color < >`)
- Ramp swaps are sampled from palette PNGs at runtime (no hardcoded color tables)
- Save/load animation project to `.ron`, including equipped layer keys and layer mapping metadata
- Backward-compatible load for legacy `.ron` files that do not contain `layers` metadata

Not implemented yet:
- Collision editor
- Anchor editor
- Advanced outfit preset management UI
- Per-part custom animation override authoring UI

## Run

```powershell
cd C:\Users\yup\rust_project\asset_hander
cargo run
```

## Validation

```powershell
cargo fmt
cargo check
cargo test
```

## Quick Workflow

1. Open a sprite sheet image under `assets/`.
2. Configure grid values in the left panel.
3. In `Animations`, choose category/clip, then arm a direction.
4. Click cells in the workspace to build/toggle the active track.
5. Use preview strip selection to edit step metadata (`Duration`, `Flip`).
6. Set playback mode for the clip.
7. In `Parts`, cycle layer selections to preview outfits and paper-doll composition.
8. In `Parts`, use each layer's `color < >` controls to override that layer's palette/variant.
9. In `Palettes`, cycle global palette ramps/variants to remap layers that are not overridden.
10. Save via `File -> Save Anim Project`, reload via `File -> Load Anim Project`.

## Layer Filename Rules

Expected format:

`fbas_<layer>_<name>_<ver+palette>[_e].png`

Example:

`fbas_14head_headscarf_00b_e.png`

Key parsing constraints:
- `fbas` base prefix required
- layer code must be one of `00undr..15over`
- version token starts with 2 digits
- optional palette suffix allowed only for version `00` (`a|b|c|d|f`)
- optional special suffix is currently only `_e`

## RON Project Notes

Project now stores:
- sheet metadata
- animation defaults
- clips/tracks/steps/playback
- `layers` metadata:
  - `equipped_parts: Vec<String>`
  - `mappings: BTreeMap<String, PartMapping>` (default `FollowBodyCells`)

Legacy projects without a `layers` block still load.

## Code Map

- `src/main.rs`: app bootstrap
- `src/app/*`: editor state resources
- `src/features/image/*`: image loading/sync
- `src/features/grid/*`: grid math and selection
- `src/features/animation/mod.rs`: animation model + RON serialization
- `src/features/layers/mod.rs`: paper-doll catalog/equip/atlas setup
- `src/ui/shell.rs`: UI layout, interactions, viewer, save/load handlers
- `src/ui/panels/*`: panel style helpers
- `src/ui/widgets/*`: splitter and scroll helpers
