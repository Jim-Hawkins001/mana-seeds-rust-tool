# Step 2 Plan - Animation Authoring Pipeline

## Scope Lock

Step 2 turns the current editor into an animation authoring tool.

Implement only:

- Animation catalog (seed list)
- Per-direction tracks (`Up`, `Down`, `Right`; `Left` derived)
- Ordered frame steps (`cell` + `ms` + `flip_x`)
- Playback behavior (`Loop`, `LoopN`, `OneShot` with hold option)
- Serialization to one RON file

Out of scope for Step 2:

- Layers
- Collision tools
- Anchor tools
- Other metadata systems beyond this animation pipeline

No feature creep: stop at the Step 2 finish line.

## Goal

Current state:

- You can load a sheet
- You can draw/select a grid

Step 2 adds:

- Clip catalog and tree selection
- Directional track editing
- Step editing with timing
- Playback mode authoring
- Save/load round-trip through a single `.ron` file

## Data Model To Store

### Project-level metadata

- `image_path` (sprite sheet path)
- `grid`:
  - `cell_w`
  - `cell_h`
  - `offset_x`
  - `offset_y`
  - `spacing_x`
  - `spacing_y`
  - `columns`

### Animation-level metadata (per clip)

- `id` (stable key, e.g. `walk`, `fish_cast`)
- `display` (UI name, e.g. `Walk`, `Fish: Cast`)
- `category` / domain (e.g. `Locomotion`, `Combat`)
- `playback`:
  - `Loop`
  - `LoopN { times }`
  - `OneShot { hold_last }`

### Track-level data (per direction)

- Authored tracks: usually `Up`, `Down`, `Right`
- `Left` is derived from `Right` by mirror rule, not authored directly

### Step-level data (per frame in a track)

- `cell: u16`
- `ms: u16`
- `flip_x: bool`

This supports:

- Non-contiguous frame ordering
- Per-frame timing
- Mirrored repeated sequences

## Recommended RON Schema

```ron
(
  version: 1,
  sheet: (
    image: "assets/body.png",
    grid: (
      cell_w: 64,
      cell_h: 64,
      offset_x: 0,
      offset_y: 0,
      spacing_x: 0,
      spacing_y: 0,
      columns: 32
    ),
  ),
  defaults: (frame_ms: 120),
  clips: [
    (
      id: "walk",
      display: "Walk",
      category: "Locomotion",
      playback: Loop,
      tracks: {
        Up: (steps: []),
        Down: (steps: []),
        Right: (steps: []),
      },
      derives: [MirrorX(from: Right, to: Left)],
    ),
  ],
)
```

Notes:

- `tracks` is a sparse map: clips may have only some directions authored
- `derives` guarantees `Left` via mirror rule without authored left steps

## Seed Catalog (Domains)

### Locomotion

- `walk`
- `run`
- `jump`
- `climb`
- `mount_up`
- `ride_mount`
- `ride_mount_drunk`

### Carry

- `pickup`
- `carry`
- `walk_carry`
- `run_carry`
- `jump_carry`
- `putdown`

### Interaction

- `push`
- `pull`
- `plant_seed`
- `water`
- `work_station`

### Performance / Social

- `sing`
- `play_guitar`
- `play_flute`
- `play_drums`
- `wave`
- `hug`
- `thumbs_up`
- `sniff`
- `sad`
- `shocked`
- `laugh`
- `impatient`
- `mad_stomp`

### Fishing

- `fish_cast`
- `fish_wait_bite`
- `fish_catch`

### Combat

- `strike_overhand`
- `strike_forehand`
- `strike_backhand`
- `bow_shot`
- `hurt`
- `evade`

### Animals

- `pet_dog`
- `milk_cow`
- `pet_horse`

### Poses

- `idle`
- `sit_floor`
- `sit_ledge`
- `sit_chair`
- `meditate`
- `sleep`
- `sleep_sit_chair`

### Drinking

- `drink_stand`
- `drink_sit`

Seed only names/default metadata, not frame content.

## Editor Workflow

1. Select clip and direction
- Tree path pattern: `Animations -> Locomotion -> Walk -> Right`
- Direction selection sets active track for editing

2. Add frames from current grid selection
- Button: `Add Selected Cells`
- Appends cells to active track
- Default ordering: by ascending cell index
- Default values: `ms = defaults.frame_ms`, `flip_x = false`

3. Append mirrored copy
- Button: `Append Mirrored Copy`
- Appends same cell order with `flip_x = true`
- Example output:
  - `(48,false),(49,false),(50,false),(48,true),(49,true),(50,true)`

4. Edit frame timing
- Step row fields: `cell | ms | flip_x`
- Inline `ms` edit
- Bulk tools:
  - `Set ms for selected steps`
  - `Set ms for all steps in track`

5. Set clip playback behavior
- Per-clip playback selector:
  - `Loop`
  - `LoopN { times }`
  - `OneShot { hold_last }`

Rule mapping:

- Stay on final frame until state changes -> `OneShot { hold_last: true }`
- Repeat forever -> `Loop`
- Repeat exact count -> `LoopN { times: N }`

6. Left direction handling
- `Left` shown in tree as derived/read-only
- Derivation rule: `MirrorX(from: Right, to: Left)`
- Optional: `Preview Left` button to inspect derived result

## Minimal UI Additions

### Left panel

- Tree view: `Category -> Clip -> Direction`
- `+ New Animation` button
- New clip defaults to empty tracks + mirror derive rule

### Clip properties panel

- Editable display name
- Playback mode and options
- Optional per-clip default frame duration override

### Track editor panel

- Steps list with visible cell indices
- `Add Selected Cells`
- `Append Mirrored Copy`
- Delete and reorder controls (up/down acceptable)
- Timing editor + bulk timing actions

## Implementation Order (Exact)

1. Create structs (`AnimProject`, `Clip`, `Track`, `Step`, `Playback`, `DeriveRule`)
2. Add RON serialize/deserialize (load/save functional before UI work)
3. Add seed catalog generation (empty clips + `Right -> Left` mirror derive default)
4. Build tree UI (`Category -> Clip -> Direction` selection)
5. Bind `Add Selected Cells` to selected track writes
6. Build steps list UI (cell indices visible immediately)
7. Add timing editor (`ms` per step + bulk tools)
8. Add `Append Mirrored Copy`
9. Add playback UI (`Loop`, `OneShot hold`, `LoopN`)
10. Stop (Step 2 complete)

## Definition of Step 2 Complete

All must be true:

- Open body sheet image
- Choose `Walk -> Right`
- Select grid cells
- Click `Add Selected Cells`
- See steps list populate with cell indices
- Edit per-frame `ms`
- Append mirrored copy for second half
- Set playback mode (`Loop` / `LoopN` / `OneShot hold`)
- Save to `.ron`
- Reload and observe identical restored state

That is the Step 2 finish line.
