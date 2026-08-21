# Tiny Settlements

A card-based ecosystem gardener: drag cards to combine species, direct your
Gardener, and cultivate the **Genesis Bloom**. Bevy port of the Godot original,
built on the [game-utils](https://github.com/mlm-games/game-utils) ecosystem and
Repose UI.

## How to Play

- **Drag cards** around the board; drop one onto another to combine (recipes) or stage it
- **Drop the Gardener** onto a seed/spore to plant it, onto a nutrient to apply it,
  onto mulch to upgrade substrate, or onto waste toxin to clean it
- Fungi produce nutrients passively; slugs eat fungi and make mulch;
  flutterwings pollinate mature vines
- Too many slugs spawn waste toxins — keep the balance
- **Win** by growing the Genesis Bloom. `Esc` pauses, `R` restarts after game over

## Features

- **Card Ecosystem** - 24 card types with planting, growth chains, passive production, pollination, hatching, and recipes
- **Game Feel** - recoil, knockback, slow-motion, rumble (gamepad)
- **Screen Effects** - trauma shake, freeze frame, flash white, chromatic aberration pulse + decay
- **Transitions** - fade to black, circle wipe scene transitions with input edge blocking
- **Audio** - channel-based SFX/Music/UI buses with independent volume control (`BaseVolume` × bus), pitch variation, pooled SFX (uses Bevy built-in audio, no external dep)
- **Localization** - Fluent-based i18n with 7 bundled locales (en, es, fr, de, ja, zh, pt), language switcher in settings, `LocaleResources` resource
- **Save System** - persistent RON save with atomic writes + version migration via `directories`
- **Object Pooling** - generic entity pool with acquire/release
- **Juice** - pop-in, squash & stretch, bounce scale, shake, particles with gravity/fade
- **VFX** - damage numbers, particle bursts, trail emitters
- **UI Effects** - hover scale, typewriter text, number counter
- **Math Utils** - smooth_damp, approach, wave (f32, Vec2, Vec3)
- **Center Pivot** - sprite origin centering component
- **UI** - animated buttons, popup system, pause/settings/credits with localized text (Repose)
- **States** - Splash -> Loading -> Title -> InGame with pause overlay
- **Theme** - centralized color constants
- **Dev Tools** - FPS overlay, state logging (dev feature)

## Quick Start

```bash
cargo run
```

With physics (bevy_rapier2d):
```bash
cargo run --features physics
```

Dev build with hot-reload:
```bash
cargo run --features dev
```

## Structure

The game-feel ecosystem (audio, transitions, juice, VFX, save, i18n, pooling, game feel)
lives in the **[game-utils](https://github.com/mlm-games/game-utils)** workspace, split into:

- `crates/game-utils` - Bevy-agnostic core (save manager, i18n, math, stats, achievements)
- `crates/game-utils-bevy/src` - Bevy plugins:
  - `audio.rs` - channel-based audio buses (SfxChannel/MusicChannel/UiChannel)
  - `center_pivot.rs` - sprite origin centering
  - `game_feel.rs` - recoil, knockback, slow-motion, gamepad rumble
  - `i18n.rs` - Fluent-based localization (7 locales, language switcher)
  - `juice.rs` - pop-in, squash/stretch, bounce, shake, particles
  - `pooling.rs` - generic entity pooling
  - `save.rs` - RON save/load with atomic writes + version migration
  - `screen_effects.rs` - trauma, freeze frame, flash white, chromatic aberration
  - `time_scale.rs` - single owner of virtual-time speed/pause
  - `transitions.rs` - fade/circle wipe with input edge blocking
  - `ui_effects.rs` - hover scale, typewriter, number counter
  - `vfx.rs` - damage numbers, particle bursts, trail emitters

This template repo holds only the app layer:

```
src/
├── main.rs              # Entry point
├── app.rs               # AppPlugin, states, system sets
├── save.rs              # SaveData type (persisted via game-utils)
├── screens/             # Splash, loading, title
├── menus/               # Main, pause, settings, credits (localized)
├── theme/               # Theme resource
├── game/                # Card ecosystem gameplay (defs + simulation)
├── dev_tools.rs         # FPS overlay, state logging
└── asset_tracking.rs    # Preload tracking
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bevy` (git rev) | Engine |
| `repose-bevy` / `repose-*` | UI framework |
| `fluent-bundle` + `unic-langid` | Localization (Fluent) |
| `serde` + `ron` + `directories` | Save system |
| `rand` | Random variation (audio pitch, VFX) |
| `bevy_rapier2d` (optional) | Physics |

## License

GPL-3.0
