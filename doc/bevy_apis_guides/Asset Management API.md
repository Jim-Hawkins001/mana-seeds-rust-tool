Asset Management API (Bevy 0.18) - Practical Guide for Top-Down Games

1) Core Asset APIs

- `AssetServer`
  - Asynchronous loading entrypoint.
  - Returns typed handles.
- `Handle<T>`
  - Lightweight reference to loaded data.
- `Assets<T>`
  - Storage of actual loaded asset values.

2) Basic Loading

```rust
fn setup(asset_server: Res<AssetServer>) {
    let player_tex: Handle<Image> = asset_server.load("textures/player.png");
    let bgm: Handle<AudioSource> = asset_server.load("audio/bgm.ogg");

    let _ = (player_tex, bgm);
}
```

Important:
- Loading is async.
- Handle creation is immediate.
- Actual data appears later in `Assets<T>`.

3) Spawning with Handles

```rust
fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("textures/player.png");
    commands.spawn(Sprite::from_image(texture));
}
```

4) Reading/Mutating Loaded Data

Use resources like `Res<Assets<Image>>` or `ResMut<Assets<Image>>`.

```rust
fn inspect_images(images: Res<Assets<Image>>, handle: Res<PlayerTexture>) {
    if let Some(image) = images.get(&handle.0) {
        let _size = image.size();
    }
}
```

5) Loading with Per-Asset Settings

`load_with_settings` lets you tweak loader behavior for specific files.

Example used by tile array textures:

```rust
let handle = asset_server.load_with_settings(
    "textures/array_texture.png",
    |settings: &mut ImageLoaderSettings| {
        settings.array_layout = Some(ImageArrayLayout::RowCount { rows: 4 });
    },
);
```

6) Pixel-Art Specific Setup

Use nearest sampling globally:

```rust
.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
```

This prevents blurry sprites when scaled.

7) Organizing Asset Handles for a Top-Down Game

Use a resource as an asset catalog.

```rust
#[derive(Resource)]
struct GameAssets {
    player: Handle<Image>,
    goblin: Handle<Image>,
    tileset: Handle<Image>,
    bgm: Handle<AudioSource>,
}
```

Then load once in startup/loading state and reuse handles.

8) Suggested Asset Folder Layout

- `assets/textures/player/`
- `assets/textures/enemies/`
- `assets/textures/tiles/`
- `assets/audio/music/`
- `assets/audio/sfx/`
- `assets/fonts/`

Consistent naming helps with deterministic lookups and content tooling.

9) State-Driven Loading Flow

Recommended states:
- `Boot`
- `Loading`
- `InGame`

Flow:
1. Enter `Loading`: create handles and store in `GameAssets` resource.
2. Poll readiness (via your chosen load-check pattern).
3. Transition to `InGame` once required assets are ready.

10) Runtime Asset Updates

During gameplay you may:
- swap sprite handles for equipment/skins
- stream chunk textures
- spawn/despawn VFX assets

Keep heavy loading outside critical simulation systems.

11) Common Asset Mistakes

- Calling `asset_server.load` repeatedly every frame.
- Assuming loaded data exists immediately after creating handle.
- Storing paths all over code instead of central asset catalog resource.
- Mixing UI/gameplay assets without naming conventions.

12) Audio API Notes (high level)

With default plugins enabled, audio assets are loaded through `AssetServer` as `Handle<AudioSource>`.
Use dedicated audio systems/resources to control music state by gameplay state.

13) Large World Advice

For open-world top-down projects:
- keep chunk metadata lightweight in ECS
- keep content assets in stable handles/resources
- stream entity presence, not entire asset sets, during movement
