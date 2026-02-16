System Syntax (Bevy 0.18+) - Practical Patterns for Top-Down Games

This file focuses on system signatures, scheduling, and execution control.

1) Basic System Signature

A system is just a function with supported system params.

```rust
fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Transform, With<Player>>,
) {
    for mut tf in &mut q {
        if input.pressed(KeyCode::KeyW) {
            tf.translation.y += 200.0 * time.delta_secs();
        }
    }
}
```

2) Registering Systems

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Update, update_system)
    .add_systems(FixedUpdate, fixed_system)
    .run();
```

- `Startup`: once
- `Update`: every frame
- `FixedUpdate`: fixed simulation timestep

3) Fixed Timestep Control

```rust
app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
```

Use `FixedUpdate` for:
- movement simulation
- hit detection
- AI decisions
- deterministic server-authoritative steps

4) System Ordering

Be explicit only where needed.

- `.before(other_system)`
- `.after(other_system)`
- `.chain()` for strict sequence
- `SystemSet` for grouped ordering

```rust
.add_systems(
    Update,
    (read_input, build_intent, apply_movement).chain(),
)
```

5) Run Conditions (`run_if`)

Gate systems by state/resources/input.

```rust
.add_systems(
    Update,
    player_input
        .run_if(in_state(GameState::InGame))
        .run_if(resource_exists::<PlayerConfig>),
)
```

Combine conditions:
- `.and(...)`
- `.or(...)`
- `not(...)`

6) State Schedules

State APIs:
- `#[derive(States)]`
- `.init_state::<GameState>()`
- `OnEnter(State::X)`
- `OnExit(State::X)`
- `in_state(State::X)`
- `ResMut<NextState<GameState>>`

Typical top-down state model:
- `Boot`
- `MainMenu`
- `Loading`
- `InGame`
- `Paused`

7) Common System Params

- `Commands`
- `Res<T>` / `ResMut<T>`
- `Query<...>`
- `Single<...>`
- `Option<Single<...>>`
- `Populated<...>`
- `Local<T>`
- `MessageReader<T>` / `MessageWriter<T>` / `MessageMutator<T>`
- `ParamSet<(...)>`

When to use:
- `Single` for one camera/player
- `Populated` to skip systems unless at least one entity exists
- `ParamSet` when borrow rules would otherwise conflict

8) Fallible Systems

Systems may return `Result`.

```rust
fn system_that_can_fail(single: Single<&Transform, With<Player>>) -> Result {
    let _ = *single;
    Ok(())
}
```

Global handling:

```rust
use bevy::ecs::error::warn;
app.set_error_handler(warn);
```

Useful for transitions and content-loading edge cases.

9) Message Flow in Systems

```rust
#[derive(Message)]
struct DealDamage { amount: i32 }

fn send(mut w: MessageWriter<DealDamage>) {
    w.write(DealDamage { amount: 10 });
}

fn recv(mut r: MessageReader<DealDamage>) {
    for msg in r.read() {
        let _ = msg.amount;
    }
}
```

If reading and writing same message type in one system:
- use `ParamSet`, or
- use `Local<MessageCursor<T>>` + `ResMut<Messages<T>>`

10) Input System Syntax (0.18)

Keyboard and mouse use `ButtonInput` resources.

```rust
fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if keys.just_pressed(KeyCode::Space) {}
    if mouse.pressed(MouseButton::Left) {}
}
```

11) Camera + Window as `Single`

```rust
fn aim_system(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (cam, cam_tf) = *camera;
    if let Some(cursor) = window.cursor_position()
        && let Ok(world_pos) = cam.viewport_to_world_2d(cam_tf, cursor)
    {
        let _ = world_pos;
    }
}
```

12) Recommended System Pipeline for Top-Down

In `FixedUpdate`:
- `read_player_input` (or cache intent in `Update`)
- `move_entities`
- `resolve_collisions`
- `apply_combat`
- `apply_deaths`

In `Update`:
- `animate_sprites`
- `camera_follow`
- `ui_refresh`
- `audio_fx`

13) Debugging and Safety Tips

- Prefer narrow queries over giant tuples.
- Use `Changed<T>` to reduce work.
- Keep strict ordering minimal.
- Fail early with fallible params instead of panicking.
- Use state-scoped entities to avoid stale world objects.
