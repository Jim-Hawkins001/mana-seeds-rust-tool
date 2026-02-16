# Step 1 Plan - UI Foundation & Grid Selection

## Scope Lock

Step 1 is intentionally limited to the foundation. Implement only:

- Upload image
- Slice grid
- Render grid overlay
- Select / multi-select cells
- Resizable + scrollable UI containers
- Mode dropdown with only Sprite Mode active

Out of scope for Step 1:

- Animation creation/playback
- Collision editor
- Anchor tools
- Metadata persistence
- File saving/export

No feature creep: anything beyond this moves to Step 2+ backlog.

## 1. High-Level Layout

```text
+------------------------------------------------------------------+
|                          TOP TOOLBAR                             |
|  File | Edit | Mode ? | (future options)                        |
+------------------------------------------------------------------+
|          |                                       |               |
|          |                                       |               |
| LEFT     |           CENTER CANVAS               | RIGHT PANEL   |
| PANEL    |        (Image + Grid Overlay)         | (Reserved)    |
|          |                                       |               |
+------------------------------------------------------------------+
|                     BOTTOM PANEL (Player Area)                   |
+------------------------------------------------------------------+
```

Final structural definition for Step 1.

## 2. Panel Responsibilities

### 2.1 Top Toolbar - Global Menu Bar

Purpose: app-level state control.

Contains:

- `File` dropdown
- `Open Image` (active)
- `Save` (future placeholder)
- `Exit`
- `Edit` (future placeholder)
- `Mode` dropdown:
- `Sprite Mode` (default/active)
- `Collision Mode` (future placeholder)
- `Anchor Mode` (future placeholder)

Step 1 behavior:

- Only Sprite Mode is active.
- Image upload via `File -> Open Image`.

### 2.2 Left Panel - Sprite Asset Panel

This panel is only for sprite sheet and slicing parameters.

Section A - Image Configuration

- Image Name (read-only)
- Image Resolution (auto-detected)
- Offset X
- Offset Y

Section B - Grid Configuration

- Rows
- Columns
- Cell Width
- Cell Height
- Apply / Regenerate Grid

Grid updates in real time when applied.

Section C - Future Placeholder

- Animations (collapsed)
- Add Animation (disabled)
- Animation List (disabled)

This panel is the Sprite Definition Layer.

### 2.3 Center Panel - Sprite Grid Canvas

Main workspace responsibilities:

- Display uploaded image
- Render grid overlay
- Index cells (row-major)
- Click to select
- Click again to deselect
- Multi-selection support
- Highlight selected cells

Visual behavior:

- Visible grid lines
- Optional index number per cell
- Selected cells show semi-transparent fill + clear border emphasis

Technical state:

```rust
struct GridState {
    rows: u32,
    columns: u32,
    cell_width: u32,
    cell_height: u32,
    offset_x: u32,
    offset_y: u32,
    selected_cells: HashSet<usize>,
}
```

Index formula:

```text
index = row * columns + column
```

Step 1 only implements visual selection logic. No animation assignment, no metadata editing.

### 2.4 Right Panel - Metadata Panel (Reserved)

Step 1:

- Empty container
- Scrollable
- Resizable
- Placeholder label: `Frame Metadata (Future)`

Future role:

- Collision shapes
- Anchor points
- Interaction zones
- Per-frame metadata

This panel is the Data Layer (for later separate file system).

### 2.5 Bottom Panel - Animation Player Panel

Step 1 purpose: layout container only.

- Scrollable
- Resizable height
- Placeholder controls only:
- Play Button (disabled)
- Frame timeline bar (disabled)
- FPS input (disabled)

No playback logic in Step 1.

## 3. Container Behavior Requirements

All major panels must:

- Be resizable via drag splitters
- Be scrollable when content exceeds bounds
- Scale cleanly with window resizing
- Never overlap or break layout

Layout strategy:

- Vertical stack:
- Top Toolbar (fixed height)
- Middle Section (flex grow)
- Bottom Panel (fixed or adjustable height)

- Middle Section:
- Horizontal flex row
- Left Panel (resizable width)
- Center Canvas (flex grow)
- Right Panel (resizable width)

## 4. Functional Scope - Step 1 Only

Fully implemented:

- Upload image
- Display image in center
- Define grid (rows/columns or cell size)
- Render grid overlay
- Click selection toggle
- Multi-selection support
- Resizable + scrollable panels
- Mode dropdown with Sprite Mode as active default

Explicitly not implemented:

- Animation creation
- Animation playback
- Frame ordering
- Collision editing
- Anchor editing
- Metadata persistence
- File saving

## 5. Conceptual File Separation (Future)

Designed now, implemented later.

File A - Sprite Definition File:

- Image path
- Grid config
- Frame count

File B - Animation Definition File:

- Animation names
- Direction mappings
- Frame index sequences

File C - Metadata File:

- Collision shapes
- Anchor points
- Interaction radii

Mapping:

- Left Panel -> File A (+ later File B)
- Right Panel -> File C

## 6. Interaction Flow - Step 1

1. Launch tool
2. `File -> Open Image`
3. Image loads in center
4. Set rows/columns or cell size
5. Click `Apply Grid`
6. Grid renders
7. Click cells to select/multi-select
8. Selected cells highlight

End of Step 1 flow.

## 7. Visual Identity Tone

- Neutral dark background
- Slight panel borders
- Subtle grid lines
- Clear selected-cell highlight
- Minimal, precise, tool-first presentation

## 8. Implementation Sequence

1. Lock scope in code comments/docs (no Step 2 features)
2. Build UI shell + split layout + scroll containers
3. Define core state structs (`GridState`, image/layout/mode state)
4. Implement Top Toolbar with `Open Image`
5. Implement Left Panel inputs + apply/regenerate grid
6. Implement Center Canvas image + grid + selection logic
7. Add Right/Bottom placeholder panels (disabled-only controls)
8. Validate resize behavior and interaction edge cases

## 9. Exit Criteria

Step 1 is complete only when:

- All in-scope features work end-to-end
- Out-of-scope features are placeholders only
- No animation/collision/anchor logic exists in active code paths
- Panel responsibilities remain clean and separated
