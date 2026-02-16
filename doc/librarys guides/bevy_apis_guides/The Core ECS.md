The Core ECS (Bevy 0.18) - Top-Down Game Focus

This guide is API-first and targets Bevy 0.18. It is written for real gameplay loops (player movement, enemies, world streaming, combat, UI).

1) Core World Model

- Entity
  - Lightweight ID with attached components.
  - Spawn with `commands.spawn((...))`.
- Component
  - Plain Rust data attached to entities.
  - `#[derive(Component)]`.
- Resource
  - Global singleton-style data.
  - `#[derive(Resource)]`, access with `Res<T>` / `ResMut<T>`.
- System
  - Function that reads/writes ECS data.
  - Registered with `.add_systems(ScheduleLabel, system_fn)`.

2) Essential ECS APIs You Will Use Constantly

- App + schedules
  - `App::new()`
  - `.add_systems(Startup, setup)`
  - `.add_systems(Update, gameplay_systems)`
  - `.add_systems(FixedUpdate, fixed_simulation_systems)`
- Spawning and editing entities
  - `commands.spawn((ComponentA, ComponentB))`
  - `commands.entity(entity).insert(ComponentX)`
  - `commands.entity(entity).remove::<ComponentX>()`
  - `commands.entity(entity).despawn()`
- Queries
  - `Query<&Transform, With<Player>>`
  - `Query<(&mut Transform, &Velocity), (With<Player>, Without<Stunned>)>`
  - `Single<&mut Transform, With<Player>>` when exactly one player is expected.

3) Top-Down Component Design Pattern

Use small components, not giant structs.

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct MoveSpeed(f32);
```

Why this shape works:
- Better query composition (movement, combat, AI can be separate systems).
- Better parallelism (fewer systems fighting over same data).
- Better change detection (`Changed<T>`).

4) Query Filters + Change Detection

- Structural filters
  - `With<T>`
  - `Without<T>`
- State-change filters
  - `Changed<T>`
  - `Added<T>`

Example: only recompute expensive path data when target changed.

```rust
fn update_paths(
    query: Query<(Entity, &Target), Changed<Target>>,
) {
    for (_entity, _target) in &query {
        // recompute path only when needed
    }
}
```

5) Commands vs Direct Access

- Use `Commands` for structural world changes (spawn/despawn/insert/remove).
- Use direct mutable query access for frame logic (`&mut Transform`, `&mut Health`, etc).

Rule of thumb:
- Structural change -> Commands
- Value update -> Query mutability

6) Relationships and Hierarchies

Bevy has built-in entity relationships and supports custom relationships.

- Built-in parent/child style hierarchy for scene-like structures.
- Custom relationships via attributes:
  - `#[relationship(relationship_target = ...)]`
  - `#[relationship_target(relationship = ...)]`

Top-down usage:
- Weapon child attached to player.
- Floating health bars attached to enemies.
- AI relationships (`Targeting(Entity)` / `TargetedBy(Vec<Entity>)`).

7) Messages (for decoupled gameplay flow)

Bevy 0.18 uses message APIs:
- Define: `#[derive(Message)] struct DamageEvent { amount: i32 }`
- Register: `.add_message::<DamageEvent>()`
- Send: `MessageWriter<DamageEvent>` + `.write(...)`
- Read: `MessageReader<DamageEvent>` + `.read()`
- Mutate in flight: `MessageMutator<DamageEvent>`

Good top-down uses:
- `DealDamage`
- `PickupCollected`
- `QuestAdvanced`
- `EnemyDied`

8) Local System State

`Local<T>` stores per-system state without creating a global resource.

Use for:
- Cooldown scratch values.
- Last frame debug counters.
- Local cursors for advanced message processing.

9) Fallible Params and Fallible Systems

Bevy 0.18 supports robust error-aware patterns:
- Fallible params: `Single`, `Option<Single<...>>`, `Populated<...>`.
- Fallible systems returning `Result`.
- Global error handler via `app.set_error_handler(...)`.

Use this for gameplay safety:
- Skip combat system if no player exists yet.
- Avoid panics during scene transitions.

10) State-Driven ECS

- Define states: `#[derive(States)] enum GameState { ... }`
- Init: `.init_state::<GameState>()`
- Transition with `NextState<GameState>`.
- Scope entities to state with:
  - `DespawnOnExit(GameState::X)`
  - `DespawnOnEnter(GameState::X)`

This keeps menus, gameplay, and pause overlays cleanly separated.

11) Practical ECS Blueprint for a Top-Down Game

Recommended modules:
- `player`: input -> movement intent
- `movement`: intent + speed -> velocity -> transform
- `camera`: follow/zoom/cursor world projection
- `combat`: damage messages + health updates
- `ai`: target selection + chase/attack states
- `loot`: drops and pickup collision
- `world`: chunk/tile streaming and spawn/despawn

Recommended schedule split:
- `Startup`: load initial world + camera + player
- `FixedUpdate`: simulation-critical logic (movement/combat/AI)
- `Update`: visuals, animation, camera smoothing, UI

12) Common Pitfalls to Avoid

- Monolithic components with too many unrelated fields.
- Mutating the same component everywhere (creates contention).
- Doing expensive work every frame instead of using `Changed<T>`.
- Mixing menu/gameplay entities without state scoping.
- Assuming exactly one entity exists without `Single` and validation.

13) Minimal ECS Skeleton

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d,));
    commands.spawn((Player, Transform::default(), Velocity::default()));
}

fn player_simulation(mut q: Query<(&mut Transform, &Velocity), With<Player>>, time: Res<Time>) {
    for (mut tf, vel) in &mut q {
        tf.translation += (vel.0 * time.delta_secs()).extend(0.0);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, player_simulation)
        .run();
}
```

14) Bevy 0.18 Note

Use the Bevy 0.18 APIs in this guide (messages, modern schedule patterns, relationships, fallible params). If you reference older tutorials, verify API names against 0.18 docs before copying.
