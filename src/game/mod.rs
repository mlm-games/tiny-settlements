mod card_defs;

use bevy::ecs::query::QueryFilter;
#[cfg(test)]
use bevy::ecs::world::CommandQueue;

pub use card_defs::*;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use game_utils_bevy::game_feel::{GameFeel, SlowMotion};
use game_utils_bevy::juice::{Juice, Particle};
use game_utils_bevy::save::SaveManager;
use game_utils_bevy::screen_effects::{
    ChromaticAberration, FlashWhite, FreezeFrame, ScreenEffects, Trauma,
};
use game_utils_bevy::transitions::Transition;
use game_utils_bevy::vfx::{DamageNumber, TrailGhost, VfxSpawner};
use rand::RngExt;

use crate::app::{AppState, Paused};
use crate::save::SaveData;

pub const CARD_SIZE: Vec2 = Vec2::new(96.0, 128.0);
pub const BOARD_MIN: Vec2 = Vec2::new(-480.0, -280.0);
pub const BOARD_MAX: Vec2 = Vec2::new(480.0, 280.0);
pub const NEARBY: f32 = 95.0;

#[derive(Component)]
pub struct GameCleanup;

#[derive(Component)]
pub struct Card {
    pub card_type: CardType,
    pub is_planted: bool,
    pub needs_pollination: bool,
    pub is_pollinated: bool,
    pub is_working: bool,
    pub action: Option<GardenerAction>,
}

#[derive(Component)]
struct CardTitle;
#[derive(Component)]
struct CardStatus;

#[derive(Component)]
struct Dragging {
    offset: Vec2,
}

#[derive(Component)]
struct WorkTimer {
    timer: Timer,
}

#[derive(Component)]
struct PassiveTimer {
    timer: Timer,
    kind: PassiveKind,
}

#[derive(Resource)]
pub struct GameSession {
    pub game_over: bool,
    pub victory: bool,
    pub end_reason: String,
    pub gardener: Option<Entity>,
    pub focus: f32,
    pub max_focus: f32,
    pub action_cost: f32,
    pub biodiversity: u32,
    pub toxins: u32,
    pub tracked: HashMap<CardType, u32>,
    pub status: String,
    pub hint: String,
    pub hint_timer: f32,
    focus_recharge: Timer,
    nutrient_spawn: Timer,
    passive_scan: Timer,
    waste_check: Timer,
    toxicity_tick: Timer,
    pub focus_recharge_rate: f32,
    pub max_slugs_before_waste: u32,
    pub max_toxins_before_loss: u32,
    /// Fire win/lose juice exactly once.
    pub end_fx_done: bool,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            game_over: false,
            victory: false,
            end_reason: String::new(),
            gardener: None,
            focus: 100.0,
            max_focus: 100.0,
            // Godot CARD_PROPERTIES uses 50; keep that for fidelity
            action_cost: 50.0,
            biodiversity: 0,
            toxins: 0,
            tracked: HashMap::new(),
            status: String::new(),
            hint: String::new(),
            hint_timer: 0.0,
            focus_recharge: Timer::from_seconds(0.5, TimerMode::Repeating),
            nutrient_spawn: Timer::from_seconds(18.0, TimerMode::Repeating),
            passive_scan: Timer::from_seconds(1.0, TimerMode::Repeating),
            waste_check: Timer::from_seconds(12.0, TimerMode::Repeating),
            toxicity_tick: Timer::from_seconds(1.5, TimerMode::Repeating),
            focus_recharge_rate: 3.0,
            max_slugs_before_waste: 5,
            max_toxins_before_loss: 6,
            end_fx_done: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct RestartFlag(pub bool);

#[derive(Resource, Default)]
struct PendingSpawns(Vec<(CardType, Vec2, bool)>);

#[derive(Resource, Default)]
struct PendingDespawns(Vec<Entity>);

#[derive(Resource, Default)]
struct PendingPassives(Vec<(Entity, PassiveKind, f32)>);

#[derive(Resource, Default)]
struct PendingWork(Vec<(Entity, f32, GardenerAction)>);

/// One-shot juice requests so systems stay small.
#[derive(Resource, Default)]
struct PendingFx(Vec<FxEvent>);

enum FxEvent {
    Craft { pos: Vec2 },
    Plant { pos: Vec2 },
    Clean { pos: Vec2 },
    Produce { pos: Vec2, color: Color },
    Win { pos: Vec2 },
    Lose,
    Toxin { pos: Vec2 },
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSession>()
            .init_resource::<RestartFlag>()
            .init_resource::<PendingSpawns>()
            .init_resource::<PendingDespawns>()
            .init_resource::<PendingPassives>()
            .init_resource::<PendingWork>()
            .init_resource::<PendingFx>()
            .add_systems(OnEnter(AppState::InGame), setup_game)
            .add_systems(OnExit(AppState::InGame), cleanup_game)
            .add_systems(
                Update,
                (
                    handle_restart_input,
                    process_restart,
                    begin_drag,
                    update_drag,
                    end_drag,
                    apply_pending_work,
                    tick_work_timers,
                    tick_passive_timers,
                    world_timers,
                    board_pressure,
                    apply_pending_spawns,
                    apply_pending_despawns,
                    end_game_fx,
                    apply_pending_fx,
                    update_card_labels,
                    tick_hint,
                    sync_hud,
                    flush_save_on_win,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame))
                    .run_if(|p: Res<Paused>| !p.0)
                    .run_if(|t: Res<Transition<AppState>>| !t.block_input),
            );
    }
}

fn setup_game(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut save: ResMut<SaveData>,
    manager: Res<SaveManager>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut pending_passive: ResMut<PendingPassives>,
    mut pending_work: ResMut<PendingWork>,
    mut pending_fx: ResMut<PendingFx>,
) {
    *session = GameSession::default();
    pending_spawn.0.clear();
    pending_despawn.0.clear();
    pending_passive.0.clear();
    pending_work.0.clear();
    pending_fx.0.clear();
    // persist immediately so quitting/losing can't lose the stat
    save.times_played = save.times_played.saturating_add(1);
    let _ = manager.save(&*save);

    commands.spawn((
        GameCleanup,
        Sprite {
            color: Color::srgb(0.10, 0.16, 0.12),
            custom_size: Some(BOARD_MAX - BOARD_MIN + Vec2::splat(48.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    spawn_card(
        &mut commands,
        &mut session,
        None,
        CardType::Gardener,
        Vec2::new(-360.0, 0.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        None,
        CardType::BioSubstrate,
        Vec2::new(-180.0, 90.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        None,
        CardType::BioSubstrate,
        Vec2::new(-180.0, -90.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        None,
        CardType::SporePod,
        Vec2::new(20.0, 90.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        None,
        CardType::NutrientSlime,
        Vec2::new(20.0, 0.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        None,
        CardType::NutrientSlime,
        Vec2::new(20.0, -90.0),
        false,
    );

    session.hint =
        "Drag Spore Pod onto Bio-Substrate, then drop Gardener on the spore to plant.".into();
    session.hint_timer = 8.0;
}

fn cleanup_game(
    mut commands: Commands,
    q: Query<Entity, With<GameCleanup>>,
    numbers: Query<Entity, With<DamageNumber>>,
    particles: Query<Entity, With<Particle>>,
    trails: Query<Entity, With<TrailGhost>>,
) {
    for e in q
        .iter()
        .chain(numbers.iter())
        .chain(particles.iter())
        .chain(trails.iter())
    {
        commands.entity(e).despawn();
    }
}

fn clamp_board(p: Vec2) -> Vec2 {
    let h = CARD_SIZE * 0.5;
    p.clamp(BOARD_MIN + h, BOARD_MAX - h)
}

fn random_board_pos() -> Vec2 {
    let mut rng = rand::rng();
    let h = CARD_SIZE * 0.5;
    Vec2::new(
        rng.random_range((BOARD_MIN.x + h.x)..(BOARD_MAX.x - h.x)),
        rng.random_range((BOARD_MIN.y + h.y)..(BOARD_MAX.y - h.y)),
    )
}

fn offset_near(origin: Vec2) -> Vec2 {
    let mut rng = rand::rng();
    clamp_board(origin + Vec2::new(rng.random_range(45.0..75.0), rng.random_range(-24.0..24.0)))
}

fn spawn_card(
    commands: &mut Commands,
    session: &mut GameSession,
    assets: Option<&AssetServer>,
    card_type: CardType,
    pos: Vec2,
    planted: bool,
) -> Option<Entity> {
    if session.game_over && card_type != CardType::GenesisBloom {
        return None;
    }
    if card_type == CardType::Gardener && session.gardener.is_some() {
        return None;
    }

    let pos = clamp_board(pos);
    let mut sprite = Sprite {
        color: card_type.color(),
        custom_size: Some(CARD_SIZE),
        ..default()
    };
    if let (Some(server), Some(path)) = (assets, card_type.asset_path()) {
        sprite.image = server.load(path);
        sprite.color = Color::WHITE;
    }

    let e = commands
        .spawn((
            GameCleanup,
            Card {
                card_type,
                is_planted: planted,
                needs_pollination: card_type == CardType::MatureVine,
                is_pollinated: false,
                is_working: false,
                action: None,
            },
            sprite,
            Transform::from_translation(pos.extend(1.0)),
        ))
        .with_children(|p| {
            p.spawn((
                CardTitle,
                Text2d::new(card_type.label()),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
                Transform::from_xyz(0.0, 42.0, 1.0),
            ));
            p.spawn((
                CardStatus,
                Text2d::new(""),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 0.85)),
                TextLayout::justify(Justify::Center),
                Transform::from_xyz(0.0, -44.0, 1.0),
            ));
            p.spawn((
                Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.12),
                    custom_size: Some(Vec2::splat(46.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 2.0, 0.5),
            ));
        })
        .id();

    Juice::pop_in(commands, e, 0.22);

    if card_type == CardType::Gardener {
        session.gardener = Some(e);
    }
    if card_type.is_mature_species() {
        let c = session.tracked.entry(card_type).or_insert(0);
        *c += 1;
        if *c == 1 {
            session.biodiversity = session.tracked.len() as u32;
        }
    }
    if card_type == CardType::GenesisBloom {
        session.game_over = true;
        session.victory = true;
        session.status = "GENESIS BLOOM CULTIVATED! The Ecosystem Thrives!".into();
    }
    Some(e)
}

fn remove_biodiversity(session: &mut GameSession, t: CardType) {
    if let Some(c) = session.tracked.get_mut(&t) {
        *c = c.saturating_sub(1);
        if *c == 0 {
            session.tracked.remove(&t);
            session.biodiversity = session.tracked.len() as u32;
        }
    }
}

// --- input / drag ---

fn pointer_world(window: &Window, cam: &Query<(&Camera, &GlobalTransform)>) -> Option<Vec2> {
    let (camera, gt) = cam.single().ok()?;
    let c = window.cursor_position()?;
    camera.viewport_to_world_2d(gt, c).ok()
}

fn hit(card: Vec2, p: Vec2) -> bool {
    let h = CARD_SIZE * 0.5;
    (p.x - card.x).abs() <= h.x && (p.y - card.y).abs() <= h.y
}

fn begin_drag(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(Entity, &mut Transform, &Card), Without<Dragging>>,
    dragging: Query<Entity, With<Dragging>>,
    session: Res<GameSession>,
) {
    if session.game_over || !mouse.just_pressed(MouseButton::Left) || !dragging.is_empty() {
        return;
    }
    let Ok(w) = window.single() else { return };
    let Some(world) = pointer_world(w, &cam) else {
        return;
    };

    let mut best: Option<(Entity, f32, f32)> = None;
    for (e, tf, card) in &cards {
        if card.is_working {
            continue;
        }
        let p = tf.translation.truncate();
        if !hit(p, world) {
            continue;
        }
        let z = tf.translation.z;
        let y = p.y;
        if best.is_none_or(|(_, bz, by)| z > bz || ((z - bz).abs() < f32::EPSILON && y > by)) {
            best = Some((e, z, y));
        }
    }
    if let Some((e, _, _)) = best
        && let Ok((_, mut tf, _)) = cards.get_mut(e)
    {
        let p = tf.translation.truncate();
        commands.entity(e).insert(Dragging { offset: p - world });
        tf.translation.z = 40.0;
    }
}

fn update_drag(
    window: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform)>,
    mut q: Query<(&Dragging, &mut Transform)>,
) {
    let Ok(w) = window.single() else { return };
    let Some(world) = pointer_world(w, &cam) else {
        return;
    };
    for (d, mut tf) in &mut q {
        let p = clamp_board(world + d.offset);
        tf.translation.x = p.x;
        tf.translation.y = p.y;
    }
}

fn end_drag(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut session: ResMut<GameSession>,
    mut cards: Query<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut pending_work: ResMut<PendingWork>,
    mut pending_fx: ResMut<PendingFx>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let dragged: Vec<Entity> = cards
        .iter()
        .filter_map(|(e, _, _, d)| d.map(|_| e))
        .collect();
    if dragged.is_empty() {
        return;
    }

    // snapshot positions/types so resolution doesn't fight the borrow checker
    let snap: Vec<(Entity, Vec2, CardType, bool, bool)> = cards
        .iter()
        .map(|(e, tf, c, _)| {
            (
                e,
                tf.translation.truncate(),
                c.card_type,
                c.is_working,
                c.is_planted,
            )
        })
        .collect();

    for src in dragged {
        let Some((_, spos, type_a, working_a, _)) = snap.iter().find(|(e, ..)| *e == src).copied()
        else {
            continue;
        };
        if working_a {
            commands.entity(src).remove::<Dragging>();
            continue;
        }

        // find overlap target (top-most by z)
        let mut overlaps: Vec<(Entity, Vec2, f32, CardType, bool, bool)> = snap
            .iter()
            .filter(|(e, pos, _, w, _)| {
                *e != src && !*w && (*pos - spos).length() < CARD_SIZE.x * 0.72
            })
            .map(|(e, pos, t, w, p)| (*e, *pos, 0.0, *t, *w, *p))
            .collect();
        for item in &mut overlaps {
            if let Ok((_, tf, _, _)) = cards.get(item.0) {
                item.2 = tf.translation.z;
            }
        }
        overlaps.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let target = overlaps.first().copied();

        if let Some((tgt, tpos, _, type_b, _, planted_b)) = target {
            resolve_drop(
                &mut session,
                &mut cards,
                &mut pending_spawn,
                &mut pending_despawn,
                &mut pending_work,
                &mut pending_fx,
                src,
                type_a,
                spos,
                tgt,
                type_b,
                tpos,
                planted_b,
            );
        }

        if let Ok((_, mut tf, _, _)) = cards.get_mut(src) {
            tf.translation.z = 1.0;
        }
        commands.entity(src).remove::<Dragging>();
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_drop(
    session: &mut GameSession,
    cards: &mut Query<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>,
    pending_spawn: &mut PendingSpawns,
    pending_despawn: &mut PendingDespawns,
    pending_work: &mut PendingWork,
    pending_fx: &mut PendingFx,
    src: Entity,
    type_a: CardType,
    spos: Vec2,
    tgt: Entity,
    type_b: CardType,
    tpos: Vec2,
    _planted_b: bool,
) {
    if session.game_over {
        return;
    }

    if type_a == CardType::Gardener {
        gardener_on(session, cards, pending_work, src, tgt, type_b, tpos);
        return;
    }

    // recipes take priority over placement hints
    if let Some(out) = recipe(type_a, type_b) {
        pending_despawn.0.push(src);
        pending_despawn.0.push(tgt);
        let mid = (spos + tpos) * 0.5;
        pending_spawn.0.push((out, mid, false));
        pending_fx.0.push(FxEvent::Craft { pos: mid });
        session.hint = format!("Crafted {}!", out.label());
        session.hint_timer = 2.5;
        return;
    }

    if type_a.is_seed_or_spore() && type_b.is_substrate() {
        if let Ok((_, mut tf, _, _)) = cards.get_mut(src) {
            tf.translation = (tpos + Vec2::new(0.0, 16.0)).extend(2.0);
        }
        session.hint = format!("Drop Gardener on {} to plant", type_a.label());
        session.hint_timer = 3.0;
        return;
    }

    if type_a.is_nutrient() {
        if let Ok((_, mut tf, _, _)) = cards.get_mut(src) {
            tf.translation = (tpos + Vec2::new(10.0, 16.0)).extend(2.0);
        }
        session.hint = format!("Drop Gardener on {} to apply", type_a.label());
        session.hint_timer = 3.0;
        return;
    }

    if type_a == CardType::RichMulch && type_b == CardType::BioSubstrate {
        if let Ok((_, mut tf, _, _)) = cards.get_mut(src) {
            tf.translation = (tpos + Vec2::new(0.0, 16.0)).extend(2.0);
        }
        session.hint = "Drop Gardener on Mulch (near substrate) to upgrade".into();
        session.hint_timer = 3.0;
    }
}

fn gardener_on(
    session: &mut GameSession,
    cards: &mut Query<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>,
    pending_work: &mut PendingWork,
    gardener: Entity,
    target: Entity,
    type_b: CardType,
    tpos: Vec2,
) {
    let cost = session.action_cost;

    // plant a seed/spore
    if type_b.is_seed_or_spore() {
        let planted = cards
            .get(target)
            .map(|(_, _, c, _)| c.is_planted)
            .unwrap_or(true);
        if planted {
            return;
        }
        if nearest(cards, target, |t| t.is_substrate()).is_none() {
            session.hint = format!("{} needs substrate nearby", type_b.label());
            session.hint_timer = 3.0;
            return;
        }
        if !spend(session, cost) {
            return;
        }
        move_gardener(cards, gardener, tpos);
        pending_work.0.push((target, 2.0, GardenerAction::Plant));
        return;
    }

    // apply a nutrient to a nearby needy plant
    if type_b.is_nutrient() {
        let Some(plant) = plant_needing(cards, target, type_b) else {
            session.hint = "No plant nearby needs this nutrient".into();
            session.hint_timer = 3.0;
            return;
        };
        // reject before spending if the plant's substrate requirement isn't met
        let plant_type = cards
            .get(plant)
            .map(|(_, _, c, _)| c.card_type)
            .unwrap_or(type_b);
        if let Some(need) = plant_type.needs_substrate()
            && nearest(cards, plant, |t| t == need).is_none()
        {
            session.hint = format!("{} needs {} nearby", plant_type.label(), need.label());
            session.hint_timer = 3.0;
            return;
        }
        if !spend(session, cost) {
            return;
        }
        let ppos = cards
            .get(plant)
            .map(|(_, tf, _, _)| tf.translation.truncate())
            .unwrap_or(tpos);
        move_gardener(cards, gardener, ppos);
        pending_work
            .0
            .push((plant, 2.0, GardenerAction::ApplyNutrient { source: target }));
        return;
    }

    // clean waste toxin
    if type_b == CardType::WasteToxin {
        if !spend(session, cost) {
            return;
        }
        move_gardener(cards, gardener, tpos);
        pending_work.0.push((target, 4.0, GardenerAction::Clean));
        return;
    }

    // upgrade substrate with mulch
    if type_b == CardType::RichMulch {
        let Some(sub) = nearest(cards, target, |t| t == CardType::BioSubstrate) else {
            session.hint = "Mulch needs Bio-Substrate nearby".into();
            session.hint_timer = 3.0;
            return;
        };
        if !spend(session, cost) {
            return;
        }
        let spos = cards
            .get(sub)
            .map(|(_, tf, _, _)| tf.translation.truncate())
            .unwrap_or(tpos);
        move_gardener(cards, gardener, spos);
        pending_work.0.push((
            sub,
            2.0,
            GardenerAction::UpgradeSubstrate { source: target },
        ));
    }
}

fn move_gardener(
    cards: &mut Query<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>,
    gardener: Entity,
    target: Vec2,
) {
    if let Ok((_, mut tf, mut c, _)) = cards.get_mut(gardener) {
        c.is_working = true;
        tf.translation = (target + Vec2::new(0.0, 56.0)).extend(30.0);
    }
}

fn spend(session: &mut GameSession, cost: f32) -> bool {
    if session.focus + 0.01 < cost {
        session.hint = "Not enough Gardener Focus!".into();
        session.hint_timer = 2.5;
        return false;
    }
    session.focus = (session.focus - cost).max(0.0);
    true
}

fn nearest(
    cards: &Query<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>,
    src: Entity,
    pred: impl Fn(CardType) -> bool,
) -> Option<Entity> {
    let spos = cards.get(src).ok()?.1.translation.truncate();
    let mut best = None;
    let mut bd = NEARBY;
    for (e, tf, c, _) in cards.iter() {
        if e == src || !pred(c.card_type) {
            continue;
        }
        let d = spos.distance(tf.translation.truncate());
        if d < bd {
            bd = d;
            best = Some(e);
        }
    }
    best
}

fn plant_needing(
    cards: &Query<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>,
    nutrient: Entity,
    ntype: CardType,
) -> Option<Entity> {
    let npos = cards.get(nutrient).ok()?.1.translation.truncate();
    let mut best = None;
    let mut bd = NEARBY;
    for (e, tf, c, _) in cards.iter() {
        if e == nutrient {
            continue;
        }
        let ok = c.card_type.is_plant() || (c.card_type.is_seed_or_spore() && c.is_planted);
        if !ok || c.card_type.needs_nutrient() != Some(ntype) {
            continue;
        }
        let d = npos.distance(tf.translation.truncate());
        if d < bd {
            bd = d;
            best = Some(e);
        }
    }
    best
}

fn recipe(a: CardType, b: CardType) -> Option<CardType> {
    use CardType::*;
    let pair = |x, y| (a == x && b == y) || (a == y && b == x);
    if (a == ProcessedNutrients && b == ProcessedNutrients)
        || (a == NutrientSlime && b == NutrientSlime)
    {
        return Some(SporePod);
    }
    if pair(BasicFungi, FertilizedVinePod) {
        return Some(SymbioticAlgae);
    }
    if pair(NutrientSlime, ProcessedNutrients) {
        return Some(VineSeed);
    }
    if pair(ProcessedNutrients, BasicFungi) {
        return Some(FlutterwingSpore);
    }
    if pair(FertilizedVinePod, FlutterwingSpore) {
        return Some(GrazingSlugEgg);
    }
    if pair(LuminaCrystal, FertilizedVinePod) {
        return Some(ApexSpore);
    }
    Option::None
}

fn apply_pending_work(
    mut commands: Commands,
    mut pending: ResMut<PendingWork>,
    mut cards: Query<&mut Card>,
    mut session: ResMut<GameSession>,
) {
    for (target, dur, action) in pending.0.drain(..) {
        if let Ok(mut c) = cards.get_mut(target) {
            c.is_working = true;
            c.action = Some(action);
            commands.entity(target).insert(WorkTimer {
                timer: Timer::from_seconds(dur, TimerMode::Once),
            });
            session.status = format!("Working... ({dur:.0}s)");
        }
    }
}

fn tick_work_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut session: ResMut<GameSession>,
    mut q: Query<(Entity, &mut Card, &mut WorkTimer, &Transform)>,
    others: Query<(Entity, &Transform, &Card), Without<WorkTimer>>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut pending_passive: ResMut<PendingPassives>,
    mut pending_fx: ResMut<PendingFx>,
) {
    let mut finished = Vec::new();
    for (e, mut card, mut wt, tf) in &mut q {
        if wt.timer.tick(time.delta()).just_finished() {
            let action = card.action.take();
            card.is_working = false;
            finished.push((
                e,
                action,
                tf.translation.truncate(),
                card.card_type,
                card.is_planted,
            ));
            commands.entity(e).remove::<WorkTimer>();
        }
    }
    if finished.is_empty() {
        return;
    }
    if let Some(g) = session.gardener {
        let gardener = g;
        commands.queue(move |world: &mut World| {
            if let Some(mut c) = world.get_mut::<Card>(gardener) {
                c.is_working = false;
            }
            if let Some(mut tf) = world.get_mut::<Transform>(gardener) {
                tf.translation.z = 1.0;
            }
        });
    }
    session.status.clear();

    for (e, action, pos, ctype, planted) in finished {
        let Some(action) = action else { continue };
        let sub_ok = substrate_ok_for(&others, e, ctype, pos);
        match action {
            GardenerAction::Plant => {
                set_planted(&mut commands, e);
                // SporePod / FertilizedVinePod auto-grow after plant
                start_growth(&mut pending_passive, e, ctype, true, false, sub_ok);
                pending_fx.0.push(FxEvent::Plant { pos });
                session.hint = format!("{} planted", ctype.label());
                session.hint_timer = 2.0;
            }
            GardenerAction::ApplyNutrient { source } => {
                if !sub_ok {
                    // substrate vanished mid-action: don't consume the nutrient
                    session.hint = format!(
                        "{} needs {} nearby",
                        ctype.label(),
                        ctype.needs_substrate().map(|t| t.label()).unwrap_or("?")
                    );
                    session.hint_timer = 3.0;
                } else {
                    pending_despawn.0.push(source);
                    start_growth(&mut pending_passive, e, ctype, planted, true, true);
                    pending_fx.0.push(FxEvent::Plant { pos });
                    session.hint = format!("{} growing...", ctype.label());
                    session.hint_timer = 2.0;
                }
            }
            GardenerAction::Clean => {
                pending_despawn.0.push(e);
                pending_fx.0.push(FxEvent::Clean { pos });
            }
            GardenerAction::UpgradeSubstrate { source } => {
                pending_despawn.0.push(source);
                pending_despawn.0.push(e);
                pending_spawn
                    .0
                    .push((CardType::FertileSubstrate, pos, false));
                pending_fx.0.push(FxEvent::Craft { pos });
            }
        }
    }
}

fn set_planted(commands: &mut Commands, e: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mut c) = world.get_mut::<Card>(e) {
            c.is_planted = true;
        }
    });
}

fn start_growth(
    pending_passive: &mut PendingPassives,
    e: Entity,
    ctype: CardType,
    planted: bool,
    nutrient_applied: bool,
    substrate_ok: bool,
) {
    let Some(dur) = ctype.growth_duration() else {
        return;
    };

    // Auto-grow stages (after plant / spawn)
    if ctype.auto_grows() {
        let ok = match ctype {
            CardType::SporePod | CardType::FertilizedVinePod => planted,
            CardType::FlutterwingLarva | CardType::GrowingApex => true,
            _ => false,
        };
        if ok && substrate_ok {
            pending_passive.0.push((e, PassiveKind::Grow, dur));
        }
        return;
    }

    // Nutrient-gated stages (VineSeed, YoungVine, FlutterwingSpore, ApexSpore)
    if nutrient_applied && substrate_ok {
        let ready = planted || ctype.is_plant();
        if ready {
            pending_passive.0.push((e, PassiveKind::Grow, dur));
        }
    }
}

fn substrate_ok_for<F: QueryFilter>(
    cards: &Query<(Entity, &Transform, &Card), F>,
    e: Entity,
    ctype: CardType,
    pos: Vec2,
) -> bool {
    match ctype.needs_substrate() {
        Some(need) => cards.iter().any(|(oe, otf, oc)| {
            oe != e && oc.card_type == need && otf.translation.truncate().distance(pos) < NEARBY
        }),
        None => true,
    }
}

fn tick_passive_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut session: ResMut<GameSession>,
    mut q: Query<(Entity, &mut Card, &mut PassiveTimer, &Transform)>,
    others: Query<(Entity, &Transform, &Card), Without<PassiveTimer>>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut pending_passive: ResMut<PendingPassives>,
    mut pending_fx: ResMut<PendingFx>,
) {
    // schedule passives queued last frame (skip entities despawned meanwhile)
    for (e, kind, dur) in pending_passive.0.drain(..) {
        if commands.get_entity(e).is_ok() {
            commands.entity(e).insert(PassiveTimer {
                timer: Timer::from_seconds(dur, TimerMode::Once),
                kind,
            });
        }
    }

    let mut done = Vec::new();
    for (e, card, mut pt, tf) in &mut q {
        if pt.timer.tick(time.delta()).just_finished() {
            done.push((
                e,
                pt.kind,
                tf.translation.truncate(),
                card.card_type,
                card.is_planted,
            ));
            commands.entity(e).remove::<PassiveTimer>();
        }
    }

    for (e, kind, pos, ctype, planted) in done {
        match kind {
            PassiveKind::Grow => {
                if let Some(next) = ctype.next_growth() {
                    pending_despawn.0.push(e);
                    let keep_planted = planted
                        || matches!(
                            next,
                            CardType::BasicFungi
                                | CardType::YoungVine
                                | CardType::MatureVine
                                | CardType::GrowingApex
                                | CardType::GenesisBloom
                        );
                    pending_spawn.0.push((next, pos, keep_planted));
                    if next == CardType::GenesisBloom {
                        session.game_over = true;
                        session.victory = true;
                        session.status = "GENESIS BLOOM CULTIVATED! The Ecosystem Thrives!".into();
                    } else {
                        pending_fx.0.push(FxEvent::Produce {
                            pos,
                            color: next.color(),
                        });
                    }
                }
            }
            PassiveKind::Produce => {
                if let Some((prod, interval)) = ctype.produces_passively() {
                    let mut ok = true;
                    if let Some(need) = ctype.needs_substrate() {
                        ok = others.iter().any(|(oe, otf, oc)| {
                            oe != e
                                && oc.card_type == need
                                && otf.translation.truncate().distance(pos) < NEARBY
                        });
                    }
                    if ok {
                        let p = offset_near(pos);
                        pending_spawn.0.push((prod, p, false));
                        pending_passive.0.push((e, PassiveKind::Produce, interval));
                        pending_fx.0.push(FxEvent::Produce {
                            pos: p,
                            color: prod.color(),
                        });
                    }
                }
            }
            PassiveKind::Pollinate => {
                commands.queue(move |world: &mut World| {
                    if let Some(mut c) = world.get_mut::<Card>(e) {
                        c.is_pollinated = true;
                        c.needs_pollination = false;
                    }
                });
                let p = pos + Vec2::new(0.0, -24.0);
                pending_spawn
                    .0
                    .push((CardType::FertilizedVinePod, p, false));
                pending_fx.0.push(FxEvent::Produce {
                    pos: p,
                    color: CardType::FertilizedVinePod.color(),
                });
            }
            PassiveKind::Hatch => {
                pending_despawn.0.push(e);
                pending_spawn.0.push((CardType::GrazingSlug, pos, false));
                pending_fx.0.push(FxEvent::Produce {
                    pos,
                    color: CardType::GrazingSlug.color(),
                });
            }
            PassiveKind::Eat => {
                if let Some(food_t) = ctype.eats()
                    && let Some((food, _, _)) = others.iter().find(|(oe, otf, oc)| {
                        *oe != e
                            && oc.card_type == food_t
                            && otf.translation.truncate().distance(pos) < NEARBY
                    })
                {
                    pending_despawn.0.push(food);
                    if let Some((_, interval)) = ctype.produces_passively() {
                        pending_passive.0.push((e, PassiveKind::Produce, interval));
                    }
                }
            }
        }
    }
}

fn world_timers(
    time: Res<Time>,
    mut session: ResMut<GameSession>,
    cards: Query<(Entity, &Transform, &Card)>,
    has_passive: Query<(), With<PassiveTimer>>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_passive: ResMut<PendingPassives>,
    mut pending_fx: ResMut<PendingFx>,
) {
    if session.game_over {
        return;
    }

    if session.focus_recharge.tick(time.delta()).just_finished() {
        let gardener_busy = session
            .gardener
            .and_then(|g| cards.get(g).ok())
            .map(|(_, _, c)| c.is_working)
            .unwrap_or(true);
        if !gardener_busy {
            session.focus = (session.focus + session.focus_recharge_rate).min(session.max_focus);
        }
    }

    if session.nutrient_spawn.tick(time.delta()).just_finished() {
        pending_spawn
            .0
            .push((CardType::NutrientSlime, random_board_pos(), false));
    }

    if session.waste_check.tick(time.delta()).just_finished() {
        let slugs = cards
            .iter()
            .filter(|(_, _, c)| c.card_type == CardType::GrazingSlug)
            .count() as u32;
        if slugs >= session.max_slugs_before_waste {
            let pos = random_board_pos();
            pending_spawn.0.push((CardType::WasteToxin, pos, false));
            pending_fx.0.push(FxEvent::Toxin { pos });
            session.hint = "Too many slugs! Waste toxin appeared.".into();
            session.hint_timer = 3.0;
        }
    }

    if !session.passive_scan.tick(time.delta()).just_finished() {
        return;
    }

    for (e, tf, c) in &cards {
        if c.is_working || has_passive.get(e).is_ok() {
            continue;
        }
        let pos = tf.translation.truncate();

        // producers (slugs only produce after eating)
        if let Some((_, interval)) = c.card_type.produces_passively()
            && c.card_type != CardType::GrazingSlug
        {
            let mut ok = true;
            if let Some(need) = c.card_type.needs_substrate() {
                ok = cards.iter().any(|(oe, otf, oc)| {
                    oe != e
                        && oc.card_type == need
                        && otf.translation.truncate().distance(pos) < NEARBY
                });
            }
            if ok {
                pending_passive.0.push((e, PassiveKind::Produce, interval));
            }
        }

        // mature vines need a nearby flutterwing to pollinate
        if c.card_type == CardType::MatureVine && c.needs_pollination && !c.is_pollinated {
            let flutter_nearby = cards.iter().any(|(oe, otf, oc)| {
                oe != e
                    && oc.card_type == CardType::MatureFlutterwing
                    && !oc.is_working
                    && otf.translation.truncate().distance(pos) < NEARBY
            });
            if flutter_nearby {
                pending_passive.0.push((e, PassiveKind::Pollinate, 5.0));
            }
        }

        // slug eggs hatch near fungi
        if c.card_type == CardType::GrazingSlugEgg
            && let Some(need) = c.card_type.needs_nearby()
        {
            let near = cards.iter().any(|(oe, otf, oc)| {
                oe != e && oc.card_type == need && otf.translation.truncate().distance(pos) < NEARBY
            });
            if near {
                pending_passive.0.push((e, PassiveKind::Hatch, 8.0));
            }
        }

        // slugs eat nearby fungi
        if let Some(food_t) = c.card_type.eats() {
            let near = cards.iter().any(|(oe, otf, oc)| {
                oe != e
                    && oc.card_type == food_t
                    && otf.translation.truncate().distance(pos) < NEARBY
            });
            if near {
                pending_passive.0.push((e, PassiveKind::Eat, 6.0));
            }
        }

        // larvae mature over time
        if c.card_type == CardType::FlutterwingLarva {
            pending_passive.0.push((e, PassiveKind::Grow, 10.0));
        }

        // final apex stage auto-matures into the Genesis Bloom
        if c.card_type == CardType::GrowingApex {
            pending_passive.0.push((e, PassiveKind::Grow, 8.0));
        }

        // planted fertilized pods grow into symbiotic algae
        if c.card_type == CardType::FertilizedVinePod && c.is_planted {
            pending_passive.0.push((e, PassiveKind::Grow, 8.0));
        }

        // planted spore that somehow lost its timer
        if c.card_type == CardType::SporePod && c.is_planted {
            pending_passive.0.push((e, PassiveKind::Grow, 5.0));
        }
    }
}

fn apply_pending_spawns(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut pending: ResMut<PendingSpawns>,
    assets: Res<AssetServer>,
) {
    for (t, pos, planted) in pending.0.drain(..) {
        spawn_card(
            &mut commands,
            &mut session,
            Some(assets.as_ref()),
            t,
            pos,
            planted,
        );
    }
}

fn apply_pending_despawns(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut pending: ResMut<PendingDespawns>,
    cards: Query<&Card>,
) {
    pending.0.sort();
    pending.0.dedup();
    for e in pending.0.drain(..) {
        if let Ok(c) = cards.get(e) {
            remove_biodiversity(&mut session, c.card_type);
            if session.gardener == Some(e) && !session.game_over {
                session.game_over = true;
                session.victory = false;
                session.end_reason = "The Gardener vanished!".into();
                session.status = "ECOSYSTEM COLLAPSED: The Gardener vanished!".into();
            }
        }
        commands.entity(e).despawn();
    }
}

fn is_toxin_vulnerable(t: CardType) -> bool {
    matches!(
        t,
        CardType::SporePod
            | CardType::BasicFungi
            | CardType::VineSeed
            | CardType::YoungVine
            | CardType::MatureVine
            | CardType::FlutterwingSpore
            | CardType::FlutterwingLarva
            | CardType::MatureFlutterwing
            | CardType::FertilizedVinePod
            | CardType::SymbioticAlgae
            | CardType::GrazingSlugEgg
            | CardType::GrazingSlug
            | CardType::ApexSpore
            | CardType::GrowingApex
    )
}

fn board_pressure(
    time: Res<Time>,
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut save: ResMut<SaveData>,
    manager: Res<SaveManager>,
    mut trauma: ResMut<Trauma>,
    mut chroma: ResMut<ChromaticAberration>,
    cards: Query<(Entity, &Transform, &Card)>,
    mut pending_despawn: ResMut<PendingDespawns>,
) {
    let toxins: Vec<(Entity, Vec2)> = cards
        .iter()
        .filter(|(_, _, c)| c.card_type == CardType::WasteToxin)
        .map(|(e, tf, _)| (e, tf.translation.truncate()))
        .collect();

    session.toxins = toxins.len() as u32;

    if session.game_over || toxins.is_empty() {
        return;
    }

    if !session.toxicity_tick.tick(time.delta()).just_finished() {
        return;
    }

    // Global pressure: toxins drain focus.
    let drain = toxins.len() as f32 * 4.0;
    session.focus = (session.focus - drain).max(0.0);

    ScreenEffects::add_trauma(&mut trauma, (0.03 * toxins.len() as f32).clamp(0.03, 0.25));
    ScreenEffects::chromatic_pulse(&mut chroma, (0.02 * toxins.len() as f32).clamp(0.02, 0.12));

    // Occasionally destroy one nearby vulnerable ecosystem card.
    if let Some((victim, pos, card_type)) = cards.iter().find_map(|(e, tf, c)| {
        if c.card_type == CardType::WasteToxin || c.card_type == CardType::Gardener {
            return None;
        }
        if !is_toxin_vulnerable(c.card_type) {
            return None;
        }
        let pos = tf.translation.truncate();
        toxins
            .iter()
            .any(|(_, tpos)| pos.distance(*tpos) < 72.0)
            .then_some((e, pos, c.card_type))
    }) {
        pending_despawn.0.push(victim);
        VfxSpawner::spawn_burst(
            &mut commands,
            pos,
            10,
            Color::srgb(0.75, 0.22, 0.25),
            (25.0, 90.0),
        );
        VfxSpawner::spawn_damage_number(&mut commands, 1, pos, Color::srgb(1.0, 0.55, 0.55));
        session.hint = format!("Waste toxin consumed {}!", card_type.label());
        session.hint_timer = 2.0;
    }

    if session.focus <= 0.0 || session.toxins >= session.max_toxins_before_loss {
        session.game_over = true;
        session.victory = false;
        session.end_reason = "Toxic overflow".into();
        session.status = "ECOSYSTEM COLLAPSED: Toxic overflow!".into();

        // loss still records your best biodiversity before the collapse
        if session.biodiversity > save.high_biodiversity {
            save.high_biodiversity = session.biodiversity;
        }
        let _ = manager.save(&*save);
    }
}

fn end_game_fx(mut session: ResMut<GameSession>, mut pending_fx: ResMut<PendingFx>) {
    if !session.game_over || session.end_fx_done {
        return;
    }
    session.end_fx_done = true;
    if session.victory {
        pending_fx.0.push(FxEvent::Win { pos: Vec2::ZERO });
    } else {
        pending_fx.0.push(FxEvent::Lose);
    }
}

fn apply_pending_fx(
    mut commands: Commands,
    mut pending: ResMut<PendingFx>,
    mut trauma: ResMut<Trauma>,
    mut flash: ResMut<FlashWhite>,
    mut freeze: ResMut<FreezeFrame>,
    mut slow: ResMut<SlowMotion>,
) {
    for ev in pending.0.drain(..) {
        match ev {
            FxEvent::Craft { pos } => {
                ScreenEffects::add_trauma(&mut trauma, 0.22);
                VfxSpawner::spawn_burst(
                    &mut commands,
                    pos,
                    12,
                    Color::srgb(0.85, 0.95, 0.55),
                    (40.0, 110.0),
                );
            }
            FxEvent::Plant { pos } => {
                ScreenEffects::add_trauma(&mut trauma, 0.12);
                VfxSpawner::spawn_burst(
                    &mut commands,
                    pos,
                    8,
                    Color::srgb(0.45, 0.85, 0.5),
                    (30.0, 80.0),
                );
            }
            FxEvent::Clean { pos } => {
                ScreenEffects::add_trauma(&mut trauma, 0.18);
                VfxSpawner::spawn_burst(
                    &mut commands,
                    pos,
                    10,
                    Color::srgb(0.7, 0.85, 0.95),
                    (35.0, 90.0),
                );
            }
            FxEvent::Produce { pos, color } => {
                VfxSpawner::spawn_burst(&mut commands, pos, 6, color, (25.0, 70.0));
            }
            FxEvent::Toxin { pos } => {
                ScreenEffects::add_trauma(&mut trauma, 0.35);
                VfxSpawner::spawn_burst(
                    &mut commands,
                    pos,
                    14,
                    Color::srgb(0.9, 0.25, 0.3),
                    (50.0, 120.0),
                );
            }
            FxEvent::Win { pos } => {
                ScreenEffects::add_trauma(&mut trauma, 0.55);
                ScreenEffects::flash_white(&mut flash, 0.18);
                ScreenEffects::freeze_frame(&mut freeze, 0.06);
                GameFeel::slow_motion(&mut slow, 0.25, 0.35);
                VfxSpawner::spawn_burst(
                    &mut commands,
                    pos,
                    28,
                    Color::srgb(0.55, 0.95, 0.75),
                    (60.0, 160.0),
                );
            }
            FxEvent::Lose => {
                ScreenEffects::add_trauma(&mut trauma, 0.7);
                ScreenEffects::flash_white(&mut flash, 0.12);
                GameFeel::slow_motion(&mut slow, 0.2, 0.25);
            }
        }
    }
}

fn update_card_labels(
    session: Res<GameSession>,
    cards: Query<(&Card, &Children, Option<&PassiveTimer>, Option<&WorkTimer>)>,
    mut texts: Query<(&mut Text2d, Option<&CardStatus>, Option<&CardTitle>)>,
) {
    for (card, children, passive, work) in &cards {
        for child in children.iter() {
            let Ok((mut text, status, title)) = texts.get_mut(child) else {
                continue;
            };
            if title.is_some() {
                **text = card.card_type.label().into();
            }
            if status.is_some() {
                let s = if card.card_type == CardType::Gardener {
                    let pct = if session.max_focus > 0.0 {
                        (session.focus / session.max_focus * 100.0) as i32
                    } else {
                        0
                    };
                    format!("Focus: {pct}%")
                } else if card.card_type == CardType::MatureVine {
                    if card.needs_pollination && !card.is_pollinated {
                        "(Needs Pollination)".into()
                    } else if card.is_pollinated {
                        "(Pollinated)".into()
                    } else {
                        String::new()
                    }
                } else if let Some(p) = passive {
                    format!("({:.0}s {:?})", p.timer.remaining_secs(), p.kind)
                } else if work.is_some() {
                    "(Working)".into()
                } else if card.is_planted && card.card_type.is_seed_or_spore() {
                    "(Planted)".into()
                } else {
                    String::new()
                };
                **text = s;
            }
        }
    }
}

fn tick_hint(time: Res<Time>, mut session: ResMut<GameSession>) {
    if session.hint_timer > 0.0 {
        session.hint_timer -= time.delta_secs();
        if session.hint_timer <= 0.0 {
            session.hint.clear();
        }
    }
}

fn sync_hud(session: Res<GameSession>, bridge: Res<crate::menus::UiBridge>) {
    let Ok(mut ui) = bridge.shared.lock() else {
        return;
    };
    ui.biodiversity = session.biodiversity;
    ui.toxins = session.toxins;
    ui.focus = session.focus;
    ui.max_focus = session.max_focus;
    ui.status_line = if !session.status.is_empty() {
        session.status.clone()
    } else {
        session.hint.clone()
    };
    ui.game_over = session.game_over;
    ui.victory = session.victory;
    ui.end_reason = session.end_reason.clone();
}

/// Flush a win to disk exactly once per victory.
fn flush_save_on_win(
    session: Res<GameSession>,
    mut save: ResMut<SaveData>,
    manager: Res<SaveManager>,
    mut saved: Local<bool>,
) {
    if !session.game_over || !session.victory {
        *saved = false;
        return;
    }
    if *saved {
        return;
    }
    *saved = true;
    save.wins = save.wins.saturating_add(1);
    if session.biodiversity > save.high_biodiversity {
        save.high_biodiversity = session.biodiversity;
    }
    let _ = manager.save(&*save);
}

fn handle_restart_input(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<GameSession>,
    mut flag: ResMut<RestartFlag>,
) {
    if session.game_over && keys.just_pressed(KeyCode::KeyR) {
        flag.0 = true;
    }
}

fn process_restart(
    mut flag: ResMut<RestartFlag>,
    mut commands: Commands,
    cleanup: Query<Entity, With<GameCleanup>>,
    numbers: Query<Entity, With<DamageNumber>>,
    particles: Query<Entity, With<Particle>>,
    trails: Query<Entity, With<TrailGhost>>,
    session: ResMut<GameSession>,
    save: ResMut<SaveData>,
    manager: Res<SaveManager>,
    pending_spawn: ResMut<PendingSpawns>,
    pending_despawn: ResMut<PendingDespawns>,
    pending_passive: ResMut<PendingPassives>,
    pending_work: ResMut<PendingWork>,
    pending_fx: ResMut<PendingFx>,
) {
    if !flag.0 {
        return;
    }
    flag.0 = false;
    for e in cleanup
        .iter()
        .chain(numbers.iter())
        .chain(particles.iter())
        .chain(trails.iter())
    {
        commands.entity(e).despawn();
    }
    setup_game(
        commands,
        session,
        save,
        manager,
        pending_spawn,
        pending_despawn,
        pending_passive,
        pending_work,
        pending_fx,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SharedUi;
    use crate::menus::UiBridge;
    use bevy::time::{TimePlugin, TimeUpdateStrategy};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Minimal headless app: no render/window plugins, fixed 250ms timestep.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<AppState>();
        app.insert_resource(Paused(false));
        app.insert_resource(Transition::<AppState>::default());
        app.insert_resource(SaveData::default());
        app.insert_resource(SaveManager::new(
            "com",
            "mlm-games",
            "tiny-settlements-test",
            "test-save.ron",
            1,
        ));
        app.insert_resource(UiBridge {
            shared: Arc::new(Mutex::new(SharedUi::default())),
            actions: Arc::new(Mutex::new(Vec::new())),
        });
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        // screen-effects / game-feel resources consumed by apply_pending_fx + board_pressure
        app.insert_resource(game_utils_bevy::screen_effects::Trauma::default());
        app.insert_resource(FlashWhite::default());
        app.insert_resource(FreezeFrame::default());
        app.insert_resource(SlowMotion::default());
        app.insert_resource(ChromaticAberration::default());
        app.add_plugins(bevy::app::TaskPoolPlugin::default());
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<Image>();
        app.add_plugins(TimePlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            250,
        )));
        app.add_plugins(GamePlugin);
        app
    }

    fn enter_game(app: &mut App) {
        app.insert_resource(NextState::Pending(AppState::InGame));
        app.update();
    }

    fn cards_of(world: &mut World, t: CardType) -> Vec<Entity> {
        world
            .query::<(Entity, &Card)>()
            .iter(world)
            .filter(|(_, c)| c.card_type == t)
            .map(|(e, _)| e)
            .collect()
    }

    fn pos_of(world: &mut World, e: Entity) -> Vec2 {
        world
            .get::<Transform>(e)
            .expect("card transform")
            .translation
            .truncate()
    }

    fn spawn_at(app: &mut App, t: CardType, pos: Vec2, planted: bool) -> Entity {
        app.world_mut()
            .resource_scope(|world: &mut World, mut session: Mut<GameSession>| {
                let mut queue = CommandQueue::default();
                let spawned = {
                    let mut commands = Commands::new(&mut queue, world);
                    spawn_card(&mut commands, session.as_mut(), None, t, pos, planted)
                        .expect("spawn succeeded")
                };
                queue.apply(world);
                spawned
            })
    }

    /// Drive resolve_drop outside a system via nested resource scopes.
    #[allow(clippy::too_many_arguments)]
    fn drop_on(
        app: &mut App,
        src: Entity,
        type_a: CardType,
        spos: Vec2,
        tgt: Entity,
        type_b: CardType,
        tpos: Vec2,
    ) {
        app.world_mut()
            .resource_scope(|world: &mut World, mut session: Mut<GameSession>| {
                world.resource_scope(|world: &mut World, mut ps: Mut<PendingSpawns>| {
                    world.resource_scope(|world: &mut World, mut pd: Mut<PendingDespawns>| {
                        world.resource_scope(|world: &mut World, mut pw: Mut<PendingWork>| {
                            world.resource_scope(|world: &mut World, mut pf: Mut<PendingFx>| {
                                let mut qstate = world.query::<(
                                    Entity,
                                    &mut Transform,
                                    &mut Card,
                                    Option<&Dragging>,
                                )>();
                                let mut q = qstate.query_mut(world);
                                resolve_drop(
                                    &mut session,
                                    &mut q,
                                    ps.as_mut(),
                                    pd.as_mut(),
                                    pw.as_mut(),
                                    pf.as_mut(),
                                    src,
                                    type_a,
                                    spos,
                                    tgt,
                                    type_b,
                                    tpos,
                                    false,
                                );
                            });
                        });
                    });
                });
            });
    }

    fn gardener_act(app: &mut App, gardener: Entity, target: Entity, type_b: CardType, tpos: Vec2) {
        app.world_mut()
            .resource_scope(|world: &mut World, mut session: Mut<GameSession>| {
                world.resource_scope(|world: &mut World, mut pw: Mut<PendingWork>| {
                    let mut qstate =
                        world.query::<(Entity, &mut Transform, &mut Card, Option<&Dragging>)>();
                    let mut q = qstate.query_mut(world);
                    gardener_on(
                        &mut session,
                        &mut q,
                        pw.as_mut(),
                        gardener,
                        target,
                        type_b,
                        tpos,
                    );
                });
            });
    }

    /// Run updates until `target` exists (or budget exhausted).
    fn wait_for(app: &mut App, target: CardType, max_updates: usize) -> bool {
        for _ in 0..max_updates {
            app.update();
            if !cards_of(app.world_mut(), target).is_empty() {
                return true;
            }
        }
        false
    }

    #[test]
    fn three_card_opening_places_smoothly() {
        let mut app = test_app();
        enter_game(&mut app);

        // opening hand: spore + substrate + gardener
        let spore = cards_of(app.world_mut(), CardType::SporePod).remove(0);
        let sub = cards_of(app.world_mut(), CardType::BioSubstrate).remove(0);
        let gardener = cards_of(app.world_mut(), CardType::Gardener).remove(0);

        // placement 1: drop the Spore Pod onto Bio-Substrate -> snaps on top
        let spos = pos_of(app.world_mut(), spore);
        let tpos = pos_of(app.world_mut(), sub);
        drop_on(
            &mut app,
            spore,
            CardType::SporePod,
            spos,
            sub,
            CardType::BioSubstrate,
            tpos,
        );
        let snapped = pos_of(app.world_mut(), spore);
        assert_eq!(
            snapped,
            tpos + Vec2::new(0.0, 16.0),
            "seed should snap above substrate"
        );

        // placement 2: drop the Gardener onto the staged seed -> plant action
        gardener_act(&mut app, gardener, spore, CardType::SporePod, snapped);
        {
            let session = app.world().resource::<GameSession>();
            assert_eq!(session.focus, 50.0, "one action cost spent");
            assert!(session.gardener.is_some());
        }
        assert!(
            pos_of(app.world_mut(), gardener).y > snapped.y,
            "gardener moves above its work target"
        );

        // card 3 in play: run the pipeline until the planted spore grows into fungi
        app.update(); // apply_pending_work inserts the WorkTimer
        assert!(
            app.world().get::<WorkTimer>(spore).is_some(),
            "plant work timer should start"
        );

        assert!(
            wait_for(&mut app, CardType::BasicFungi, 80),
            "planted spore should grow into Basic Fungi"
        );

        let world = app.world_mut();
        assert!(
            cards_of(world, CardType::SporePod).is_empty(),
            "old spore card despawned"
        );
        let fungi = cards_of(world, CardType::BasicFungi).remove(0);
        let fpos = pos_of(world, fungi);
        assert!(
            (fpos - snapped).length() < 1.0,
            "fungi replaces the spore in place"
        );
        assert_eq!(
            world.resource::<GameSession>().biodiversity,
            1,
            "first mature species tracked"
        );
        assert!(
            world.resource::<GameSession>().focus >= 49.0,
            "focus recharges while gardener is idle"
        );
    }

    #[test]
    fn nutrient_pair_crafts_spore_pod() {
        let mut app = test_app();
        enter_game(&mut app);

        let before = cards_of(app.world_mut(), CardType::SporePod).len();
        let mut slimes = cards_of(app.world_mut(), CardType::NutrientSlime);
        assert_eq!(slimes.len(), 2);
        let a = slimes.remove(0);
        let b = slimes.remove(0);

        drop_on(
            &mut app,
            a,
            CardType::NutrientSlime,
            Vec2::ZERO,
            b,
            CardType::NutrientSlime,
            Vec2::new(40.0, 0.0),
        );

        // one frame flushes pending spawn/despawn through the real chain
        app.update();

        let world = app.world_mut();
        assert!(
            cards_of(world, CardType::NutrientSlime).is_empty(),
            "both ingredients consumed"
        );
        assert_eq!(
            cards_of(world, CardType::SporePod).len(),
            before + 1,
            "recipe produced a Spore Pod"
        );
    }

    #[test]
    fn vine_seed_grows_after_feeding() {
        let mut app = test_app();
        enter_game(&mut app);

        let _seed = spawn_at(
            &mut app,
            CardType::VineSeed,
            Vec2::new(-300.0, -200.0),
            true,
        );
        let nutrient = spawn_at(
            &mut app,
            CardType::ProcessedNutrients,
            Vec2::new(-260.0, -200.0),
            false,
        );
        let gardener = cards_of(app.world_mut(), CardType::Gardener).remove(0);

        // drop gardener onto the nutrient next to the seed
        let npos = pos_of(app.world_mut(), nutrient);
        gardener_act(
            &mut app,
            gardener,
            nutrient,
            CardType::ProcessedNutrients,
            npos,
        );
        assert_eq!(app.world().resource::<GameSession>().focus, 50.0);

        assert!(
            wait_for(&mut app, CardType::YoungVine, 60),
            "fed VineSeed must become YoungVine"
        );
        assert!(cards_of(app.world_mut(), CardType::VineSeed).is_empty());

        // feed again -> MatureVine (mature species, raises biodiversity)
        let vine = cards_of(app.world_mut(), CardType::YoungVine).remove(0);
        let vpos = pos_of(app.world_mut(), vine);
        let food = spawn_at(
            &mut app,
            CardType::ProcessedNutrients,
            vpos + Vec2::new(40.0, 0.0),
            false,
        );
        let fpos = pos_of(app.world_mut(), food);
        gardener_act(&mut app, gardener, food, CardType::ProcessedNutrients, fpos);

        assert!(
            wait_for(&mut app, CardType::MatureVine, 60),
            "fed YoungVine must become MatureVine"
        );
        assert_eq!(app.world().resource::<GameSession>().biodiversity, 1);
    }

    #[test]
    fn apex_spore_requires_fertile_substrate() {
        let mut app = test_app();
        enter_game(&mut app);

        let _spore = spawn_at(&mut app, CardType::ApexSpore, Vec2::new(300.0, 200.0), true);
        let crystal = spawn_at(
            &mut app,
            CardType::LuminaCrystal,
            Vec2::new(340.0, 200.0),
            false,
        );
        let gardener = cards_of(app.world_mut(), CardType::Gardener).remove(0);

        // no FertileSubstrate nearby -> feeding rejected before spending anything
        let cpos = pos_of(app.world_mut(), crystal);
        gardener_act(&mut app, gardener, crystal, CardType::LuminaCrystal, cpos);
        {
            let session = app.world().resource::<GameSession>();
            assert_eq!(session.focus, 100.0, "no focus spent on rejected action");
            assert!(
                session.hint.to_lowercase().contains("substrate"),
                "hint explains blocker"
            );
        }
        assert!(
            app.world().get::<Card>(crystal).is_some(),
            "nutrient not consumed on rejected action"
        );
        assert!(!wait_for(&mut app, CardType::GrowingApex, 40));

        // place fertile substrate nearby -> now it grows through to the bloom win
        spawn_at(
            &mut app,
            CardType::FertileSubstrate,
            Vec2::new(360.0, 220.0),
            false,
        );
        let crystal2 = spawn_at(
            &mut app,
            CardType::LuminaCrystal,
            Vec2::new(320.0, 180.0),
            false,
        );
        let c2pos = pos_of(app.world_mut(), crystal2);
        gardener_act(&mut app, gardener, crystal2, CardType::LuminaCrystal, c2pos);

        assert!(wait_for(&mut app, CardType::GrowingApex, 60));
        assert!(
            wait_for(&mut app, CardType::GenesisBloom, 80),
            "GrowingApex auto-matures into Genesis Bloom"
        );

        let session = app.world().resource::<GameSession>();
        assert!(
            session.game_over && session.victory,
            "win condition reached"
        );
        assert_eq!(
            app.world().resource::<SaveData>().wins,
            1,
            "win flushed to save"
        );
    }

    #[test]
    fn toxin_pileup_collapses_ecosystem() {
        let mut app = test_app();
        enter_game(&mut app);

        let threshold = app.world().resource::<GameSession>().max_toxins_before_loss;
        for i in 0..threshold {
            spawn_at(
                &mut app,
                CardType::WasteToxin,
                Vec2::new(-400.0 + 60.0 * i as f32, 200.0),
                false,
            );
        }

        // pressure ticks every 1.5s; loss fires on the first tick past the threshold
        let mut collapsed = false;
        for _ in 0..20 {
            app.update();
            if app.world().resource::<GameSession>().game_over {
                collapsed = true;
                break;
            }
        }
        assert!(collapsed, "collapse triggers");

        let session = app.world().resource::<GameSession>();
        assert!(!session.victory, "collapse is a loss");
        assert!(
            session.end_reason.contains("Toxic"),
            "end reason explains the loss"
        );
        assert!(session.focus < 100.0, "toxins drained focus");
        // loss still persists stats
        let save = app.world().resource::<SaveData>();
        assert!(save.times_played >= 1, "session stat persisted");
    }

    #[test]
    fn restart_after_game_over_resets_board() {
        let mut app = test_app();
        enter_game(&mut app);

        let threshold = app.world().resource::<GameSession>().max_toxins_before_loss;
        for i in 0..threshold {
            spawn_at(
                &mut app,
                CardType::WasteToxin,
                Vec2::new(-400.0 + 60.0 * i as f32, 200.0),
                false,
            );
        }
        for _ in 0..20 {
            app.update();
            if app.world().resource::<GameSession>().game_over {
                break;
            }
        }
        assert!(app.world().resource::<GameSession>().game_over);

        // trigger restart via the same flag the UI/R key sets
        app.world_mut().resource_mut::<RestartFlag>().0 = true;
        app.update();

        let world = app.world_mut();
        assert!(!world.resource::<GameSession>().game_over, "fresh session");
        assert_eq!(
            cards_of(world, CardType::SporePod).len(),
            1,
            "opening spore back"
        );
        assert!(world.resource::<GameSession>().gardener.is_some());
    }
}
