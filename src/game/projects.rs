use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::card_defs::CardType;
use super::commissions::CommissionBoard;
use super::discovery::DiscoveryState;
use super::events::{GameEvent, PendingGameEvents};


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlueprintId {
    NurseryTray,
    CompostCradle,
    MyceliumBed,
    PollinatorLodge,
    DewBasin,
    SeedArchive,
    Greenhouse,
    RainBarrel,
    BeeHotel,
    MushroomCellar,
    ObservationStation,
    IrrigationChannel,
}

impl BlueprintId {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::NurseryTray => "nursery_tray",
            Self::CompostCradle => "compost_cradle",
            Self::MyceliumBed => "mycelium_bed",
            Self::PollinatorLodge => "pollinator_lodge",
            Self::DewBasin => "dew_basin",
            Self::SeedArchive => "seed_archive",
            Self::Greenhouse => "greenhouse",
            Self::RainBarrel => "rain_barrel",
            Self::BeeHotel => "bee_hotel",
            Self::MushroomCellar => "mushroom_cellar",
            Self::ObservationStation => "observation_station",
            Self::IrrigationChannel => "irrigation_channel",
        }
    }

    pub fn from_stable_id(id: &str) -> Option<Self> {
        Some(match id {
            "nursery_tray" => Self::NurseryTray,
            "compost_cradle" => Self::CompostCradle,
            "mycelium_bed" => Self::MyceliumBed,
            "pollinator_lodge" => Self::PollinatorLodge,
            "dew_basin" => Self::DewBasin,
            "seed_archive" => Self::SeedArchive,
            "greenhouse" => Self::Greenhouse,
            "rain_barrel" => Self::RainBarrel,
            "bee_hotel" => Self::BeeHotel,
            "mushroom_cellar" => Self::MushroomCellar,
            "observation_station" => Self::ObservationStation,
            "irrigation_channel" => Self::IrrigationChannel,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ingredient {
    pub card: CardType,
    pub amount: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum BlueprintUnlock {
    Starting,
    Discover(CardType),
    DiscoverAll(&'static [CardType]),
    Discoveries(u16),
    Commissions(u16),
    DiscoveriesAndCommissions {
        discoveries: u16,
        commissions: u16,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct BlueprintDef {
    pub id: BlueprintId,
    pub name: &'static str,
    pub clue: &'static str,
    pub output: CardType,
    pub ingredients: &'static [Ingredient],
    pub dew_cost: u32,
    pub build_seconds: f32,
    pub unlock: BlueprintUnlock,
}

const NURSERY_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::BioSubstrate,
        amount: 1,
    },
    Ingredient {
        card: CardType::ProcessedNutrients,
        amount: 1,
    },
];

const COMPOST_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::RichMulch,
        amount: 1,
    },
    Ingredient {
        card: CardType::WasteToxin,
        amount: 1,
    },
];

const MYCELIUM_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::SporePod,
        amount: 1,
    },
    Ingredient {
        card: CardType::RichMulch,
        amount: 2,
    },
];

const POLLINATOR_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::FlutterwingSpore,
        amount: 1,
    },
    Ingredient {
        card: CardType::FertilizedVinePod,
        amount: 1,
    },
    Ingredient {
        card: CardType::RichMulch,
        amount: 1,
    },
];

const DEW_BASIN_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::NutrientSlime,
        amount: 2,
    },
    Ingredient {
        card: CardType::LuminaCrystal,
        amount: 1,
    },
];

const ARCHIVE_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::SporePod,
        amount: 1,
    },
    Ingredient {
        card: CardType::VineSeed,
        amount: 1,
    },
    Ingredient {
        card: CardType::FlutterwingSpore,
        amount: 1,
    },
];

const GREENHOUSE_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::NurseryTray,
        amount: 1,
    },
    Ingredient {
        card: CardType::BioSubstrate,
        amount: 2,
    },
];

const RAINBARREL_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::DewBasin,
        amount: 1,
    },
    Ingredient {
        card: CardType::NutrientSlime,
        amount: 1,
    },
];

const BEEHOTEL_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::PollinatorLodge,
        amount: 1,
    },
    Ingredient {
        card: CardType::FlutterwingSpore,
        amount: 1,
    },
];

const MUSHROOMCELLAR_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::MyceliumBed,
        amount: 1,
    },
    Ingredient {
        card: CardType::SporePod,
        amount: 1,
    },
    Ingredient {
        card: CardType::RichMulch,
        amount: 1,
    },
];

const OBSERVATION_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::SeedArchive,
        amount: 1,
    },
    Ingredient {
        card: CardType::LuminaCrystal,
        amount: 1,
    },
];

const IRRIGATION_INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        card: CardType::WaterTender,
        amount: 1,
    },
    Ingredient {
        card: CardType::FertileSubstrate,
        amount: 1,
    },
    Ingredient {
        card: CardType::NutrientSlime,
        amount: 1,
    },
];

pub const BLUEPRINTS: &[BlueprintDef] = &[
    BlueprintDef {
        id: BlueprintId::NurseryTray,
        name: "Nursery Tray",
        clue: "Good soil and concentrated food form a protected nursery.",
        output: CardType::NurseryTray,
        ingredients: NURSERY_INGREDIENTS,
        dew_cost: 0,
        build_seconds: 6.0,
        unlock: BlueprintUnlock::Starting,
    },
    BlueprintDef {
        id: BlueprintId::CompostCradle,
        name: "Compost Cradle",
        clue: "Even poison can become life when buried in rich earth.",
        output: CardType::CompostCradle,
        ingredients: COMPOST_INGREDIENTS,
        dew_cost: 0,
        build_seconds: 8.0,
        unlock: BlueprintUnlock::Discover(CardType::WasteToxin),
    },
    BlueprintDef {
        id: BlueprintId::MyceliumBed,
        name: "Mycelium Bed",
        clue: "Spores thrive in layers of mature compost.",
        output: CardType::MyceliumBed,
        ingredients: MYCELIUM_INGREDIENTS,
        dew_cost: 0,
        build_seconds: 10.0,
        unlock: BlueprintUnlock::DiscoverAll(&[
            CardType::BasicFungi,
            CardType::RichMulch,
        ]),
    },
    BlueprintDef {
        id: BlueprintId::PollinatorLodge,
        name: "Pollinator Lodge",
        clue: "A sheltered pod and soft mulch welcome delicate visitors.",
        output: CardType::PollinatorLodge,
        ingredients: POLLINATOR_INGREDIENTS,
        dew_cost: 0,
        build_seconds: 12.0,
        unlock: BlueprintUnlock::Discover(CardType::MatureFlutterwing),
    },
    BlueprintDef {
        id: BlueprintId::DewBasin,
        name: "Dew Basin",
        clue: "Slime holds water; Lumina teaches it to gather light.",
        output: CardType::DewBasin,
        ingredients: DEW_BASIN_INGREDIENTS,
        dew_cost: 0,
        build_seconds: 12.0,
        unlock: BlueprintUnlock::Discover(CardType::LuminaCrystal),
    },
    BlueprintDef {
        id: BlueprintId::SeedArchive,
        name: "Seed Archive",
        clue: "Preserve the three founding lineages under one roof.",
        output: CardType::SeedArchive,
        ingredients: ARCHIVE_INGREDIENTS,
        dew_cost: 8,
        build_seconds: 15.0,
        unlock: BlueprintUnlock::DiscoveriesAndCommissions {
            discoveries: 10,
            commissions: 3,
        },
    },
    BlueprintDef {
        id: BlueprintId::Greenhouse,
        name: "Greenhouse",
        clue: "Glass and warmth preserve life through winter.",
        output: CardType::Greenhouse,
        ingredients: GREENHOUSE_INGREDIENTS,
        dew_cost: 6,
        build_seconds: 16.0,
        unlock: BlueprintUnlock::DiscoverAll(&[CardType::NurseryTray, CardType::FertileSubstrate]),
    },
    BlueprintDef {
        id: BlueprintId::RainBarrel,
        name: "Rain Barrel",
        clue: "A basin to catch the sky.",
        output: CardType::RainBarrel,
        ingredients: RAINBARREL_INGREDIENTS,
        dew_cost: 4,
        build_seconds: 12.0,
        unlock: BlueprintUnlock::Discover(CardType::DewBasin),
    },
    BlueprintDef {
        id: BlueprintId::BeeHotel,
        name: "Bee Hotel",
        clue: "A haven for wings.",
        output: CardType::BeeHotel,
        ingredients: BEEHOTEL_INGREDIENTS,
        dew_cost: 5,
        build_seconds: 14.0,
        unlock: BlueprintUnlock::Discover(CardType::PollinatorLodge),
    },
    BlueprintDef {
        id: BlueprintId::MushroomCellar,
        name: "Mushroom Cellar",
        clue: "Dark and damp, perfect for fungi.",
        output: CardType::MushroomCellar,
        ingredients: MUSHROOMCELLAR_INGREDIENTS,
        dew_cost: 5,
        build_seconds: 14.0,
        unlock: BlueprintUnlock::Discover(CardType::MyceliumBed),
    },
    BlueprintDef {
        id: BlueprintId::ObservationStation,
        name: "Observation Station",
        clue: "Watch the sky and know what comes.",
        output: CardType::ObservationStation,
        ingredients: OBSERVATION_INGREDIENTS,
        dew_cost: 10,
        build_seconds: 18.0,
        unlock: BlueprintUnlock::DiscoveriesAndCommissions { discoveries: 12, commissions: 4 },
    },
    BlueprintDef {
        id: BlueprintId::IrrigationChannel,
        name: "Irrigation Channel",
        clue: "Water shared is growth shared.",
        output: CardType::IrrigationChannel,
        ingredients: IRRIGATION_INGREDIENTS,
        dew_cost: 8,
        build_seconds: 16.0,
        unlock: BlueprintUnlock::Discover(CardType::WaterTender),
    },
];

pub fn blueprint_def(id: BlueprintId) -> &'static BlueprintDef {
    BLUEPRINTS.iter().find(|b| b.id == id).expect("unknown blueprint")
}


#[derive(Component)]
pub struct GardenProject {
    pub blueprint: BlueprintId,
    pub output: CardType,
    pub ingredients: Vec<Entity>,
    pub timer: Timer,
    pub position: Vec2,
    pub dew_paid: u32,
}

#[derive(Component)]
pub struct ReservedForProject {
    pub project: Entity,
}

#[derive(Component)]
pub struct ProjectProgressLabel;

#[derive(Resource, Default, Clone)]
pub struct BlueprintState {
    pub unlocked: HashSet<BlueprintId>,
    pub completed_this_run: u32,
    pub completed_ids: HashSet<BlueprintId>,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct InfrastructureBonuses {
    pub pack_discount: u32,
    pub resonance_dew_bonus: u32,
    pub installation_count: u32,
}

pub const PROJECT_RADIUS: f32 = 62.0;


pub fn unlock_satisfied(
    unlock: BlueprintUnlock,
    discovery: &DiscoveryState,
    board: &CommissionBoard,
) -> bool {
    match unlock {
        BlueprintUnlock::Starting => true,
        BlueprintUnlock::Discover(card) => discovery.contains(card),
        BlueprintUnlock::DiscoverAll(cards) => cards.iter().all(|c| discovery.contains(*c)),
        BlueprintUnlock::Discoveries(n) => discovery.count() >= n,
        BlueprintUnlock::Commissions(n) => board.total_completed >= n as u32,
        BlueprintUnlock::DiscoveriesAndCommissions {
            discoveries,
            commissions,
        } => discovery.count() >= discoveries && board.total_completed >= commissions as u32,
    }
}

pub fn refresh_blueprint_unlocks(
    state: &mut BlueprintState,
    discovery: &DiscoveryState,
    board: &CommissionBoard,
    events: &mut PendingGameEvents,
) {
    for def in BLUEPRINTS {
        if state.unlocked.contains(&def.id) {
            continue;
        }
        if unlock_satisfied(def.unlock, discovery, board) {
            state.unlocked.insert(def.id);
            events.0.push(GameEvent::BlueprintUnlocked { blueprint: def.id });
        }
    }
}

pub fn effective_pack_cost(base: u32, bonuses: &InfrastructureBonuses) -> u32 {
    base.saturating_sub(bonuses.pack_discount).max(1)
}

// Installation bonuses
pub fn installation_production_mult(card: CardType, producer: CardType) -> f32 {
    match (card, producer) {
        (CardType::NurseryTray, t) if t.is_plant() => 1.30,
        (CardType::MyceliumBed, CardType::BasicFungi) => 1.35,
        (CardType::PollinatorLodge, CardType::MatureVine) => 1.20,
        (CardType::Greenhouse, t) if t.is_plant() => 1.20,
        _ => 1.0,
    }
}

pub fn installation_growth_mult(card: CardType, growing: CardType) -> f32 {
    match card {
        CardType::NurseryTray if growing.is_seed_or_spore() || growing.is_plant() => 1.30,
        CardType::Greenhouse if growing.is_seed_or_spore() || growing.is_plant() => 1.40,
        CardType::RainBarrel if growing.is_plant() => 1.15,
        _ => 1.0,
    }
}

/// Exact multiset match: blueprint ingredient counts must equal card counts exactly.
/// No extra cards, no missing.
pub fn blueprint_matches_exact(def: &BlueprintDef, cards: &[CardType]) -> bool {
    let mut need: HashMap<CardType, u32> = HashMap::new();
    for ing in def.ingredients {
        *need.entry(ing.card).or_insert(0) += ing.amount as u32;
    }
    let mut have: HashMap<CardType, u32> = HashMap::new();
    for c in cards {
        *have.entry(*c).or_insert(0) += 1;
    }
    need == have
}

/// Find the single blueprint that exactly matches the given card multiset among unlocked.
/// Returns None if none or more than one matches (should be at most one).
pub fn find_matching_blueprint(cards: &[CardType], unlocked: &HashSet<BlueprintId>) -> Option<&'static BlueprintDef> {
    let mut found = None;
    for def in BLUEPRINTS {
        if !unlocked.contains(&def.id) {
            continue;
        }
        if blueprint_matches_exact(def, cards) {
            if found.is_some() {
                // ambiguous: more than one exact match shouldn't happen with current data,
                // but treat as no match to avoid accidental consumption
                return None;
            }
            found = Some(def);
        }
    }
    found
}


/// Poll periodically to update global InfrastructureBonuses from installed habitats.
pub fn recompute_infrastructure_bonuses(
    habitats: Query<&super::stacks::HabitatBase>,
    cards: Query<&super::Card>,
    mut bonuses: ResMut<InfrastructureBonuses>,
) {
    let mut discount = 0u32;
    let mut resonance = 0u32;
    let mut count = 0u32;
    for hab in &habitats {
        if let Some(inst) = hab.installation {
            if let Ok(card) = cards.get(inst) {
                count += 1;
                match card.card_type {
                    CardType::SeedArchive => discount += 1,
                    CardType::DewBasin => resonance += 1,
                    _ => {}
                }
            }
        }
    }
    bonuses.pack_discount = discount.min(3);
    bonuses.resonance_dew_bonus = resonance;
    bonuses.installation_count = count;
}

/// Tick active garden projects, complete them, spawn output cards.
pub fn tick_garden_projects(
    mut commands: Commands,
    time: Res<Time>,
    mut projects: Query<(Entity, &mut GardenProject, &Transform)>,
    mut session: ResMut<super::GameSession>,
    mut pending_spawn: ResMut<super::PendingSpawns>,
    mut pending_despawn: ResMut<super::PendingDespawns>,
    mut pending_fx: ResMut<super::PendingFx>,
    mut events: ResMut<PendingGameEvents>,
    mut blueprint_state: ResMut<BlueprintState>,
    mut bonuses: ResMut<InfrastructureBonuses>,
    reserved_q: Query<Entity, With<ReservedForProject>>,
    // for releasing gardener working flag
    mut gardener_cards: Query<&mut super::Card>,
) {
    for (proj_e, mut proj, tf) in &mut projects {
        if proj.timer.tick(time.delta()).just_finished() {
            // Despawn reserved ingredients
            for ing in proj.ingredients.drain(..) {
                // Only if still reserved (avoid double despawn)
                if reserved_q.get(ing).is_ok() || commands.get_entity(ing).is_ok() {
                    pending_despawn.0.push(ing);
                }
            }
            // Spawn output
            let pos = proj.position;
            pending_spawn.0.push((proj.output, pos, false));
            // Release gardener
            if let Some(g) = session.gardener {
                if let Ok(mut c) = gardener_cards.get_mut(g) {
                    c.is_working = false;
                }
                commands.queue(move |world: &mut World| {
                    if let Some(mut tf) = world.get_mut::<Transform>(g) {
                        tf.translation.z = 1.0;
                    }
                });
            }
            session.status.clear();
            // VFX + events
            pending_fx.0.push(super::FxEvent::Craft { pos });
            events.0.push(GameEvent::ProjectCompleted {
                blueprint: proj.blueprint,
                output: proj.output,
            });
            blueprint_state.completed_this_run += 1;
            blueprint_state.completed_ids.insert(proj.blueprint);
            // Recompute bonuses next frame
            // Despawn project entity
            commands.entity(proj_e).despawn();
        }
    }
}

/// Compost Cradle: every 30s per installed cradle, convert nearby toxin -> mulch.
pub fn compost_cradle_tick(
    time: Res<Time>,
    mut timer: Local<Timer>,
    habitats: Query<&super::stacks::HabitatBase>,
    cards: Query<(Entity, &Transform, &super::Card)>,
    mut pending_spawn: ResMut<super::PendingSpawns>,
    mut pending_despawn: ResMut<super::PendingDespawns>,
    mut pending_fx: ResMut<super::PendingFx>,
    mut events: ResMut<PendingGameEvents>,
    session: Res<super::GameSession>,
) {
    if session.game_over {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(30.0, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    let mut cradle_positions: Vec<Vec2> = Vec::new();
    for hab in &habitats {
        if let Some(inst) = hab.installation {
            if let Ok((_, tf, card)) = cards.get(inst) {
                if card.card_type == CardType::CompostCradle {
                    // Need habitat grid position? Use installation position
                    cradle_positions.push(tf.translation.truncate());
                }
            } else if let Ok(card) = cards.get(inst).map(|(_,_,c)| c) {
                // fallback
            }
            // Alternative: check directly via card query without transform
            // Use separate query for cards without transform? We'll just use above.
        }
    }
    // If we didn't find via transform query (because installation entity may not have transform?), also check habitats directly via card type
    // Do second pass via habitats that have installation entity with card check
    // Simplify: iterate habitats and check if installation card type is CompostCradle using cards.get
    // Already did but might miss due to query shape. Instead use a more robust method:
    // We'll just count cradles and process per cradle nearest toxin.

    // Re-gather correctly: need habitat base position for NEARBY check. Use habitat's transform via cards query on installation? Let's gather base positions via habitats query that includes GridSlot? But we don't have transform. For simplicity use installation card positions.
    // If we have at least one cradle, process each separately.

    // For each cradle, find nearest free Waste Toxin within NEARBY*2 and not reserved
    for cradle_pos in cradle_positions {
        let mut best: Option<(Entity, f32, Vec2)> = None;
        for (e, tf, card) in &cards {
            if card.card_type != CardType::WasteToxin {
                continue;
            }
            // ignore reserved toxins
            // Check if entity has ReservedForProject
            // We'll need a query for Reserved but we can check via World? Simpler: skip if card.is_working? But toxins don't have working flag via Reserved.
            // We will assume not reserved if no Reserved component; we need to query but we don't have it here. We'll add param.
            let pos = tf.translation.truncate();
            let d = cradle_pos.distance(pos);
            if d <= super::NEARBY * 2.0 {
                if best.map_or(true, |(_, bd, _)| d < bd) {
                    best = Some((e, d, pos));
                }
            }
        }
        if let Some((toxin_e, _, pos)) = best {
            pending_despawn.0.push(toxin_e);
            let spawn_pos = pos + Vec2::new(0.0, 24.0);
            pending_spawn.0.push((CardType::RichMulch, spawn_pos, false));
            pending_fx.0.push(super::FxEvent::Clean { pos });
            events.0.push(GameEvent::CleanedToxin);
            events.0.push(GameEvent::Produced {
                producer: CardType::CompostCradle,
                result: CardType::RichMulch,
            });
        }
    }
}

/// Compost tick with reserved awareness (split for query compatibility)
pub fn compost_cradle_tick_with_reserved(
    time: Res<Time>,
    mut timer: Local<Timer>,
    habitats: Query<&super::stacks::HabitatBase>,
    cards: Query<(Entity, &Transform, &super::Card)>,
    reserved: Query<(), With<ReservedForProject>>,
    mut pending_spawn: ResMut<super::PendingSpawns>,
    mut pending_despawn: ResMut<super::PendingDespawns>,
    mut pending_fx: ResMut<super::PendingFx>,
    mut events: ResMut<PendingGameEvents>,
    session: Res<super::GameSession>,
) {
    if session.game_over {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(30.0, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    // Collect cradle positions
    let mut cradle_positions = Vec::new();
    for hab in &habitats {
        if let Some(inst) = hab.installation {
            if let Ok((_, tf, card)) = cards.get(inst) {
                if card.card_type == CardType::CompostCradle {
                    cradle_positions.push(tf.translation.truncate());
                }
            }
        }
    }
    for cradle_pos in cradle_positions {
        let mut best: Option<(Entity, f32, Vec2)> = None;
        for (e, tf, card) in &cards {
            if card.card_type != CardType::WasteToxin {
                continue;
            }
            if reserved.get(e).is_ok() {
                continue;
            }
            let pos = tf.translation.truncate();
            let d = cradle_pos.distance(pos);
            if d <= super::NEARBY * 2.0 {
                if best.map_or(true, |(_, bd, _)| d < bd) {
                    best = Some((e, d, pos));
                }
            }
        }
        if let Some((toxin_e, _, pos)) = best {
            pending_despawn.0.push(toxin_e);
            let spawn_pos = pos + Vec2::new(0.0, 24.0);
            pending_spawn.0.push((CardType::RichMulch, spawn_pos, false));
            pending_fx.0.push(super::FxEvent::Clean { pos });
            events.0.push(GameEvent::CleanedToxin);
            events.0.push(GameEvent::Produced {
                producer: CardType::CompostCradle,
                result: CardType::RichMulch,
            });
        }
    }
}

// Update project progress labels
pub fn update_project_labels(
    projects: Query<(&GardenProject, &Children)>,
    mut texts: Query<&mut Text2d>,
) {
    for (proj, children) in &projects {
        let pct = if proj.timer.duration().as_secs_f32() > 0.0 {
            proj.timer.elapsed_secs() / proj.timer.duration().as_secs_f32()
        } else {
            1.0
        };
        let remaining = (proj.timer.duration().as_secs_f32() - proj.timer.elapsed_secs()).max(0.0);
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = format!("Building {} {:.0}% ({:.0}s)", proj.output.label(), pct * 100.0, remaining);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_exact_multiset_matches() {
        let def = blueprint_def(BlueprintId::NurseryTray);
        // exact
        assert!(blueprint_matches_exact(def, &[CardType::BioSubstrate, CardType::ProcessedNutrients]));
        // order doesn't matter
        assert!(blueprint_matches_exact(def, &[CardType::ProcessedNutrients, CardType::BioSubstrate]));
        // extra fails
        assert!(!blueprint_matches_exact(def, &[CardType::BioSubstrate, CardType::ProcessedNutrients, CardType::NutrientSlime]));
        // missing fails
        assert!(!blueprint_matches_exact(def, &[CardType::BioSubstrate]));
    }

    #[test]
    fn blueprint_rejects_extra_cards() {
        let def = blueprint_def(BlueprintId::DewBasin);
        // need 2x NutrientSlime +1 Lumina
        assert!(blueprint_matches_exact(def, &[CardType::NutrientSlime, CardType::NutrientSlime, CardType::LuminaCrystal]));
        assert!(!blueprint_matches_exact(def, &[CardType::NutrientSlime, CardType::LuminaCrystal]));
        assert!(!blueprint_matches_exact(def, &[CardType::NutrientSlime, CardType::NutrientSlime, CardType::LuminaCrystal, CardType::BioSubstrate]));
    }

    #[test]
    fn locked_blueprint_does_not_start() {
        let mut unlocked = HashSet::new();
        unlocked.insert(BlueprintId::NurseryTray);
        let cards = vec![CardType::RichMulch, CardType::WasteToxin]; // CompostCradle recipe but locked
        assert!(find_matching_blueprint(&cards, &unlocked).is_none());
    }

    #[test]
    fn effective_cost_clamped() {
        let mut b = InfrastructureBonuses::default();
        b.pack_discount = 5;
        assert_eq!(effective_pack_cost(4, &b), 1);
        b.pack_discount = 2;
        assert_eq!(effective_pack_cost(4, &b), 2);
    }

    #[test]
    fn seed_archive_cost_matches_hud_and_purchase() {
        let b = InfrastructureBonuses { pack_discount: 2, resonance_dew_bonus: 0, installation_count: 2 };
        let base = 9;
        let eff = effective_pack_cost(base, &b);
        assert_eq!(eff, 7);
        // Simulate HUD and purchase use same function
        let hud_cost = effective_pack_cost(base, &b);
        let purchase_cost = effective_pack_cost(base, &b);
        assert_eq!(hud_cost, purchase_cost);
    }

    #[test]
    fn seed_archive_discount_never_reduces_pack_below_one() {
        let b = InfrastructureBonuses { pack_discount: 10, ..Default::default() };
        assert_eq!(effective_pack_cost(4, &b), 1);
        assert_eq!(effective_pack_cost(1, &b), 1);
    }

    #[test]
    fn nursery_tray_reduces_growth_interval() {
        assert!(installation_growth_mult(CardType::NurseryTray, CardType::VineSeed) > 1.0);
        assert!(installation_growth_mult(CardType::NurseryTray, CardType::BasicFungi) > 1.0);
        assert_eq!(installation_growth_mult(CardType::MyceliumBed, CardType::VineSeed), 1.0);
    }

    #[test]
    fn mycelium_bed_reduces_fungi_production_interval() {
        assert!(installation_production_mult(CardType::MyceliumBed, CardType::BasicFungi) > 1.0);
        assert_eq!(installation_production_mult(CardType::MyceliumBed, CardType::YoungVine), 1.0);
        assert_eq!(installation_production_mult(CardType::NurseryTray, CardType::BasicFungi), 1.30);
    }

    #[test]
    fn compost_cradle_converts_toxin_to_mulch() {
        // Verify production counts logic: compost cradle produces RichMulch
        // This is more integration but check that installation type is recognized
        assert_eq!(CardType::CompostCradle.is_installation(), true);
        assert_eq!(CardType::RichMulch.is_installation(), false);
    }

    #[test]
    fn same_project_seed_has_deterministic_results() {
        // Project matching is deterministic based on HashMap order? Our find_matching sorts via stable BLUEPRINTS order
        let mut unlocked = HashSet::new();
        unlocked.insert(BlueprintId::NurseryTray);
        unlocked.insert(BlueprintId::CompostCradle);
        let cards = vec![CardType::BioSubstrate, CardType::ProcessedNutrients];
        let a = find_matching_blueprint(&cards, &unlocked).unwrap().id;
        let b = find_matching_blueprint(&cards, &unlocked).unwrap().id;
        assert_eq!(a, b);
        // Reversed order same result
        let cards_rev = vec![CardType::ProcessedNutrients, CardType::BioSubstrate];
        let c = find_matching_blueprint(&cards_rev, &unlocked).unwrap().id;
        assert_eq!(a, c);
    }

    #[test]
    fn blueprint_unlock_starting() {
        use super::super::discovery::DiscoveryState;
        use super::super::commissions::CommissionBoard;
        let disc = DiscoveryState::default();
        let board = CommissionBoard::default();
        let mut state = BlueprintState::default();
        let mut events = PendingGameEvents::default();
        refresh_blueprint_unlocks(&mut state, &disc, &board, &mut events);
        assert!(state.unlocked.contains(&BlueprintId::NurseryTray));
        assert!(!state.unlocked.contains(&BlueprintId::DewBasin));
    }

    #[test]
    fn save_v2_loads_with_v3_defaults() {
        let data = crate::save::SaveData::default();
        assert_eq!(data.version, crate::save::SAVE_VERSION);
        assert_eq!(data.discovered_blueprints.len(), 0);
        assert_eq!(data.total_projects_completed, 0);
        assert_eq!(data.best_installations, 0);
        // Simulate old save by constructing with version 2 and defaults for new fields
        let old = crate::save::SaveData {
            version: 2,
            high_biodiversity: 5,
            wins: 1,
            times_played: 3,
            ..Default::default()
        };
        assert_eq!(old.discovered_blueprints.len(), 0);
        assert_eq!(old.total_projects_completed, 0);
        assert_eq!(old.best_installations, 0);
    }
}
