2D World and Rendering (Bevy 0.18) - Top-Down Game APIs

This guide covers rendering and world APIs that matter for a 2D top-down game.

1) 2D Bootstrapping

```rust
App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_systems(Startup, setup)
    .run();

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

- `Camera2d` is the default 2D camera component.
- `ImagePlugin::default_nearest()` is important for crisp pixel art.

2) Sprite APIs

- Single image sprite:
  - `Sprite::from_image(handle)`
- Atlas sprite:
  - `Sprite::from_atlas_image(texture_handle, TextureAtlas { ... })`

```rust
commands.spawn(Sprite::from_image(asset_server.load("textures/player.png")));
```

3) Transform and Layering (Top-Down)

Bevy 2D uses `Transform` in 3D space:
- X: left/right
- Y: down/up
- Z: draw depth

Top-down layering pattern:
- Ground: low Z (e.g. 0.0)
- Characters: mid Z (e.g. 10.0)
- FX/UI-world markers: high Z (e.g. 20.0+)

For pseudo Y-sort, derive Z from Y:

```rust
transform.translation.z = -transform.translation.y * 0.001;
```

4) Camera World Conversion APIs

For aiming and click-to-move, convert cursor to world space:

- `Camera::viewport_to_world_2d(...)`
- `Camera::world_to_viewport(...)`

```rust
if let Some(cursor) = window.cursor_position()
    && let Ok(world) = camera.viewport_to_world_2d(camera_tf, cursor)
{
    // world is Vec2 in your game world
}
```

5) Camera Motion and Zoom

Access projection through `Projection` and edit orthographic scale.

```rust
if let Projection::Orthographic(ortho) = &mut *projection {
    ortho.scale *= 0.98;
}
```

Use smooth follow in `Update`, but simulation movement in `FixedUpdate`.

6) Sprite Sheet Animation APIs

Core types used in 0.18 examples:
- `TextureAtlasLayout`
- `TextureAtlas`
- `Sprite::from_atlas_image`
- `Timer` + `TimerMode::Repeating`

Pattern:
- Store first/last frame indices in component.
- Tick timer.
- Update `sprite.texture_atlas.as_mut().index`.

7) Mesh2d for Simple Geometry

You can mix sprites with 2D meshes for debug/world markers.

- `Mesh2d(...)`
- `MeshMaterial2d(...)`
- primitives like `Rectangle`

Useful for:
- hitbox debug visuals
- minimap markers
- procedural indicators

8) Tile and Chunk Rendering

Bevy 0.18 includes tilemap chunk rendering APIs in sprite rendering:
- `TilemapChunk`
- `TilemapChunkTileData`
- `TileData`

Key methods:
- `calculate_tile_transform(tile_pos)`
- `tile_data_from_tile_pos(chunk_size, tile_pos)`

These support large map sections rendered efficiently.

9) World Streaming Pattern for Large Top-Down Maps

Recommended runtime shape:
- Keep player chunk coordinate in resource.
- Spawn chunk entities around player (N-radius).
- Despawn chunks that leave retention radius.
- Use fixed-size chunk tiles for predictable memory behavior.

You can build this yourself with ECS + chunk components, or integrate a chunk plugin.

10) UI in 2D Games (Quick API Notes)

Bevy UI in 0.18 uses ECS components like:
- `Text`
- `Node`
- `TextFont`
- `TextColor`

For top-down games:
- screen-space HUD in UI tree
- world-space labels as sprite/text entities

11) Performance Rules for 2D Rendering

- Avoid per-frame asset loads.
- Use atlases for animation-heavy entities.
- Minimize per-frame structural changes (spawn/despawn storms).
- Separate simulation (`FixedUpdate`) and visuals (`Update`).
- Use `Changed<T>` for expensive recalculations.

12) Example: Minimal Top-Down Scene Setup

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(assets.load("textures/player.png")),
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));

    commands.spawn((
        Sprite::from_image(assets.load("textures/ground.png")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
```

13) Suggested Render Layers for Top-Down

- Z 0-5: terrain
- Z 6-15: props/characters
- Z 16-25: projectiles/fx
- Z 30+: debug markers

Keep the scheme consistent project-wide.
