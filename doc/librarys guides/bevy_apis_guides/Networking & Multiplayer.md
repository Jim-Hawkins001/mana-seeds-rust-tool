Networking and Multiplayer (Bevy 0.18 Context)

Important first principle:
- Bevy itself is an engine/framework and does not provide a complete built-in high-level multiplayer gameplay stack.
- You compose networking using ecosystem crates and integrate through Bevy ECS schedules/resources/messages/components.

1) What Bevy Gives You for Multiplayer Architecture

Bevy-native building blocks:
- ECS (`Component`, `Resource`, `System`) for world state
- `FixedUpdate` for deterministic simulation step
- `Time<Fixed>` for step control
- state management (`States`, `OnEnter`, `OnExit`, `NextState`)
- messaging (`MessageWriter`, `MessageReader`) for decoupled game logic

These are the core APIs you wire networking into.

2) Recommended Simulation Split (Server-Authoritative)

- `FixedUpdate`
  - process input commands
  - simulate movement/combat
  - produce authoritative state
- `Update`
  - interpolation/extrapolation for rendering
  - camera/UI/audio response

This split avoids frame-rate-dependent gameplay bugs.

3) Core Multiplayer Data Model in ECS

Typical components/resources:
- `NetworkId` component (entity identity across peers)
- `Replicated` marker component
- `Predicted` / `Interpolated` markers
- `LocalPlayer` marker
- transport resource (client/server socket/session)
- tick resource (`u32` simulation tick)

4) Input and Command Pipeline Pattern

Client:
1. sample local input
2. package as command with tick
3. send to server
4. predict locally

Server:
1. receive command
2. validate
3. apply in fixed simulation
4. replicate results

5) Reconciliation Pattern

When authoritative state arrives:
- compare server tick state against predicted local state
- correct divergence
- optionally re-simulate buffered local inputs

Bevy ECS makes this practical by snapshotting relevant components per tick.

6) Interpolation Pattern for Remote Entities

Store last/next snapshots and render between them in `Update`.

Per entity data often includes:
- previous transform
- target transform
- interpolation alpha

Never drive remote visuals directly from network packet arrival times.

7) Message APIs vs Network Packets

Do not couple gameplay systems directly to socket code.

Use adapter systems:
- packet -> ECS message (`MessageWriter<NetInputReceived>`)
- ECS gameplay outcome -> packet (`MessageReader<ReplicateEntityState>`)

This keeps gameplay logic testable and offline-friendly.

8) State Management for Session Lifecycle

Example states:
- `MainMenu`
- `Connecting`
- `Lobby`
- `InGame`
- `Disconnected`

Use `OnEnter/OnExit` to initialize or tear down networking resources and session entities.

9) Ecosystem Crates Commonly Used with Bevy 0.18

Common choices:
- `bevy_replicon` (high-level replication pattern)
- `renet` (transport/session building block)
- `lightyear` (prediction/reconciliation focused stack)

Choose based on required control level:
- fastest bootstrap: higher-level replication crate
- maximum control: lower-level transport + custom replication logic

10) Practical Strategy for Your Project

If your game starts as host-player and later moves to dedicated servers:
- keep gameplay systems transport-agnostic
- isolate transport into plugin/module boundary
- define protocol types independent of concrete socket layer
- keep deterministic simulation in `FixedUpdate`

This minimizes rewrite cost during infra transition.

11) Security and Trust Rules

- Never trust client-reported hit/position blindly.
- Validate commands server-side.
- Keep authoritative health/inventory/state server-owned.
- Use deterministic checks where possible.

12) Testing Multiplayer Logic in Bevy

- run headless simulation ticks for deterministic tests
- test command replay and reconciliation drift
- test packet loss/latency simulation in dev builds

13) What to Avoid

- Putting gameplay authority in rendering systems.
- Mixing network IO and gameplay mutation in same large system.
- Using frame `Update` for authoritative simulation.
- Hard-coding transport details inside gameplay modules.

14) Bottom Line

For a top-down game on Bevy 0.18:
- Bevy gives excellent ECS/time/state primitives.
- Multiplayer quality depends on your simulation model and crate integration.
- Build around fixed-step authoritative simulation from day one.
