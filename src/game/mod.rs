mod card_defs;

pub use card_defs::*;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use game_utils_bevy::juice::Juice;
use game_utils_bevy::save::SaveManager;
use game_utils_bevy::transitions::Transition;
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
    pub tracked: HashMap<CardType, u32>,
    pub status: String,
    pub hint: String,
    pub hint_timer: f32,
    focus_recharge: Timer,
    nutrient_spawn: Timer,
    passive_scan: Timer,
    waste_check: Timer,
    pub focus_recharge_rate: f32,
    pub max_slugs_before_waste: u32,
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
            tracked: HashMap::new(),
            status: String::new(),
            hint: String::new(),
            hint_timer: 0.0,
            focus_recharge: Timer::from_seconds(0.5, TimerMode::Repeating),
            nutrient_spawn: Timer::from_seconds(18.0, TimerMode::Repeating),
            passive_scan: Timer::from_seconds(1.0, TimerMode::Repeating),
            waste_check: Timer::from_seconds(12.0, TimerMode::Repeating),
            focus_recharge_rate: 3.0,
            max_slugs_before_waste: 5,
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

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSession>()
            .init_resource::<RestartFlag>()
            .init_resource::<PendingSpawns>()
            .init_resource::<PendingDespawns>()
            .init_resource::<PendingPassives>()
            .init_resource::<PendingWork>()
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
                    apply_pending_spawns,
                    apply_pending_despawns,
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
) {
    *session = GameSession::default();
    save.times_played = save.times_played.saturating_add(1);

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
        CardType::Gardener,
        Vec2::new(-360.0, 0.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        CardType::BioSubstrate,
        Vec2::new(-180.0, 90.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        CardType::BioSubstrate,
        Vec2::new(-180.0, -90.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        CardType::SporePod,
        Vec2::new(20.0, 90.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        CardType::NutrientSlime,
        Vec2::new(20.0, 0.0),
        false,
    );
    spawn_card(
        &mut commands,
        &mut session,
        CardType::NutrientSlime,
        Vec2::new(20.0, -90.0),
        false,
    );

    session.hint =
        "Drag Spore Pod onto Bio-Substrate, then drop Gardener on the spore to plant.".into();
    session.hint_timer = 8.0;
}

fn cleanup_game(mut commands: Commands, q: Query<Entity, With<GameCleanup>>) {
    for e in &q {
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
    clamp_board(origin + Vec2::new(
        rng.random_range(45.0..75.0),
        rng.random_range(-24.0..24.0),
    ))
}

fn spawn_card(
    commands: &mut Commands,
    session: &mut GameSession,
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
            Sprite {
                color: card_type.color(),
                custom_size: Some(CARD_SIZE),
                ..default()
            },
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
    let Some(world) = pointer_world(w, &cam) else { return };

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
    if let Some((e, _, _)) = best {
        if let Ok((_, mut tf, _)) = cards.get_mut(e) {
            let p = tf.translation.truncate();
            commands.entity(e).insert(Dragging { offset: p - world });
            tf.translation.z = 40.0;
        }
    }
}

fn update_drag(
    window: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform)>,
    mut q: Query<(&Dragging, &mut Transform)>,
) {
    let Ok(w) = window.single() else { return };
    let Some(world) = pointer_world(w, &cam) else { return };
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
        let Some((_, spos, type_a, working_a, _)) =
            snap.iter().find(|(e, ..)| *e == src).copied()
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
        pending_spawn.0.push((out, (spos + tpos) * 0.5, false));
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
    mut gq: Query<(&mut Card, &mut Transform), Without<WorkTimer>>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut pending_passive: ResMut<PendingPassives>,
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
        if let Ok((mut gc, mut gtf)) = gq.get_mut(g) {
            gc.is_working = false;
            gtf.translation.z = 1.0;
        }
    }
    session.status.clear();

    for (e, action, pos, ctype, planted) in finished {
        let Some(action) = action else { continue };
        match action {
            GardenerAction::Plant => {
                set_planted(&mut commands, e);
                start_growth(&mut pending_passive, e, ctype, true, true);
            }
            GardenerAction::ApplyNutrient { source } => {
                pending_despawn.0.push(source);
                start_growth(&mut pending_passive, e, ctype, planted, true);
            }
            GardenerAction::Clean => pending_despawn.0.push(e),
            GardenerAction::UpgradeSubstrate { source } => {
                pending_despawn.0.push(source);
                pending_despawn.0.push(e);
                pending_spawn.0.push((CardType::FertileSubstrate, pos, false));
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
    nutrient_ok: bool,
) {
    // planted spores sprout on their own
    if ctype == CardType::SporePod && planted {
        pending_passive.0.push((e, PassiveKind::Grow, 5.0));
        return;
    }
    if ctype == CardType::FlutterwingLarva {
        pending_passive.0.push((e, PassiveKind::Grow, 10.0));
        return;
    }
    if nutrient_ok && matches!(ctype, CardType::ApexSpore | CardType::SymbioticAlgae) {
        pending_passive.0.push((e, PassiveKind::Grow, 8.0));
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
) {
    // schedule passives queued last frame
    for (e, kind, dur) in pending_passive.0.drain(..) {
        commands.entity(e).insert(PassiveTimer {
            timer: Timer::from_seconds(dur, TimerMode::Once),
            kind,
        });
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
                        session.status =
                            "GENESIS BLOOM CULTIVATED! The Ecosystem Thrives!".into();
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
                        pending_spawn.0.push((prod, offset_near(pos), false));
                        pending_passive.0.push((e, PassiveKind::Produce, interval));
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
                pending_spawn.0.push((
                    CardType::FertilizedVinePod,
                    pos + Vec2::new(0.0, -24.0),
                    false,
                ));
            }
            PassiveKind::Hatch => {
                pending_despawn.0.push(e);
                pending_spawn.0.push((CardType::GrazingSlug, pos, false));
            }
            PassiveKind::Eat => {
                if let Some(food_t) = ctype.eats() {
                    if let Some((food, _, _)) = others.iter().find(|(oe, otf, oc)| {
                        *oe != e
                            && oc.card_type == food_t
                            && otf.translation.truncate().distance(pos) < NEARBY
                    }) {
                        pending_despawn.0.push(food);
                        if let Some((_, interval)) = ctype.produces_passively() {
                            pending_passive.0.push((e, PassiveKind::Produce, interval));
                        }
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
            pending_spawn
                .0
                .push((CardType::WasteToxin, random_board_pos(), false));
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
        if let Some((_, interval)) = c.card_type.produces_passively() {
            if c.card_type != CardType::GrazingSlug {
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
        if c.card_type == CardType::GrazingSlugEgg {
            if let Some(need) = c.card_type.needs_nearby() {
                let near = cards.iter().any(|(oe, otf, oc)| {
                    oe != e
                        && oc.card_type == need
                        && otf.translation.truncate().distance(pos) < NEARBY
                });
                if near {
                    pending_passive.0.push((e, PassiveKind::Hatch, 8.0));
                }
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
    }
}

fn apply_pending_spawns(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut pending: ResMut<PendingSpawns>,
) {
    for (t, pos, planted) in pending.0.drain(..) {
        spawn_card(&mut commands, &mut session, t, pos, planted);
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
    session: ResMut<GameSession>,
    save: ResMut<SaveData>,
) {
    if !flag.0 {
        return;
    }
    flag.0 = false;
    for e in &cleanup {
        commands.entity(e).despawn();
    }
    setup_game(commands, session, save);
}
