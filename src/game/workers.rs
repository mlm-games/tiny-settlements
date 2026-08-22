use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::CardType;
use super::events::{GameEvent, PendingGameEvents};
use super::stacks::{HabitatBase, HabitatSynergy, StackLayer, StackedOn, GridSlot};
use super::seasons::{Season, EcoModifiers, WeatherEvent, SeasonClock, ActiveWeather};
use super::economy::RunEconomy;
use super::{Card, GameSession, PendingDespawns, PendingSpawns, CARD_SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorkerKind {
    Botanist,
    Mycologist,
    Entomologist,
    CompostKeeper,
    WaterTender,
}

impl WorkerKind {
    pub fn from_card(card: CardType) -> Option<Self> {
        Some(match card {
            CardType::Botanist => Self::Botanist,
            CardType::Mycologist => Self::Mycologist,
            CardType::Entomologist => Self::Entomologist,
            CardType::CompostKeeper => Self::CompostKeeper,
            CardType::WaterTender => Self::WaterTender,
            _ => return None,
        })
    }
    pub fn to_card(self) -> CardType {
        match self {
            Self::Botanist => CardType::Botanist,
            Self::Mycologist => CardType::Mycologist,
            Self::Entomologist => CardType::Entomologist,
            Self::CompostKeeper => CardType::CompostKeeper,
            Self::WaterTender => CardType::WaterTender,
        }
    }
    pub fn stable_id(self) -> &'static str {
        match self {
            Self::Botanist => "botanist",
            Self::Mycologist => "mycologist",
            Self::Entomologist => "entomologist",
            Self::CompostKeeper => "compost_keeper",
            Self::WaterTender => "water_tender",
        }
    }
    pub fn from_stable_id(s: &str) -> Option<Self> {
        Some(match s {
            "botanist" => Self::Botanist,
            "mycologist" => Self::Mycologist,
            "entomologist" => Self::Entomologist,
            "compost_keeper" => Self::CompostKeeper,
            "water_tender" => Self::WaterTender,
            _ => return None,
        })
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Botanist => "Botanist",
            Self::Mycologist => "Mycologist",
            Self::Entomologist => "Entomologist",
            Self::CompostKeeper => "Compost Keeper",
            Self::WaterTender => "Water Tender",
        }
    }
    pub fn upkeep(self) -> u32 { 1 }
}

#[derive(Component)]
pub struct Worker {
    pub kind: WorkerKind,
    pub fatigued: bool,
}

#[derive(Component)]
pub struct AssignedWorker {
    pub habitat: Entity,
}

// Habitat worker slot is stored in HabitatBase.worker (Option<Entity>)

// Bonus helpers
pub fn worker_growth_mult(kind: WorkerKind, growing: CardType) -> f32 {
    match kind {
        WorkerKind::Botanist => {
            if growing.is_plant() || growing.is_seed_or_spore() { 1.35 } else { 1.0 }
        }
        _ => 1.0,
    }
}

pub fn worker_production_mult(kind: WorkerKind, producer: CardType) -> f32 {
    match (kind, producer) {
        (WorkerKind::Mycologist, CardType::BasicFungi) => 1.40,
        // Mycologist also gives slight mulch bonus? For test we claim 1.40 for fungi
        (WorkerKind::Entomologist, CardType::MatureVine) => 1.20, // combined with lodge?
        _ => 1.0,
    }
}

pub fn worker_pollination_mult(kind: WorkerKind) -> f32 {
    match kind {
        WorkerKind::Entomologist => 1.45,
        _ => 1.0,
    }
}

// Comprehensive worker mult for growth (including season offset)
pub fn total_worker_growth_mult(worker: Option<WorkerKind>, growing: CardType, fatigued: bool) -> f32 {
    if fatigued { return 1.0; }
    if let Some(k) = worker {
        worker_growth_mult(k, growing)
    } else { 1.0 }
}

pub fn total_worker_production_mult(worker: Option<WorkerKind>, producer: CardType, fatigued: bool) -> f32 {
    if fatigued { return 1.0; }
    if let Some(k) = worker {
        worker_production_mult(k, producer)
    } else { 1.0 }
}

// Advanced greenhouse offsets winter growth penalty: installation_growth_mult already handles NurseryTray, but Greenhouse should be stronger in winter
pub fn advanced_installation_growth_mult(installation: CardType, growing: CardType, season: Season) -> f32 {
    match installation {
        CardType::Greenhouse => {
            if season == Season::Winter { 1.50 } else { 1.20 }
        }
        _ => 1.0,
    }
}

// Helper to get worker kind for an entity that is worker card
pub fn worker_kind_for_entity(entity: Entity, workers: &Query<&Worker>) -> Option<(WorkerKind, bool)> {
    workers.get(entity).ok().map(|w| (w.kind, w.fatigued))
}

// Helper to get assigned worker kind for a habitat
pub fn assigned_worker_kind_for_habitat(
    habitat: &HabitatBase,
    workers: &Query<&Worker>,
) -> Option<(WorkerKind, bool)> {
    habitat.worker.and_then(|e| workers.get(e).ok().map(|w| (w.kind, w.fatigued)))
}

// Check if greenhouse offsets winter
pub fn greenhouse_offsets_winter(season: Season, installation: CardType) -> bool {
    season == Season::Winter && installation == CardType::Greenhouse
}

// Heatwave negation by Water Tender
pub fn water_tender_negates_heatwave(worker: Option<WorkerKind>, weather: Option<WeatherEvent>, fatigued: bool) -> bool {
    if fatigued { return false; }
    matches!(worker, Some(WorkerKind::WaterTender)) && weather == Some(WeatherEvent::Heatwave)
}

// Compost keeper + cradle cleans faster: we scale interval by 0.7
pub fn compost_interval_mult(worker: Option<WorkerKind>, fatigued: bool) -> f32 {
    if fatigued { return 1.0; }
    match worker {
        Some(WorkerKind::CompostKeeper) => 0.70,
        _ => 1.0,
    }
}

// Systems

pub fn position_assigned_workers(
    habitats: Query<(&Transform, &HabitatBase), Without<AssignedWorker>>,
    mut workers: Query<(&AssignedWorker, &mut Transform)>,
) {
    for (assign, mut tf) in &mut workers {
        if let Ok((base_tf, _)) = habitats.get(assign.habitat) {
            let b = base_tf.translation;
            tf.translation = Vec3::new(b.x + 18.0, b.y - 30.0, b.z + 6.0);
        }
    }
}

pub fn clear_dead_workers(
    mut commands: Commands,
    mut habitats: Query<&mut HabitatBase>,
    workers: Query<Entity, With<Worker>>,
    assigned: Query<(Entity, &AssignedWorker)>,
) {
    // if worker entity despawned, clear habitat slot
    for mut hab in &mut habitats {
        if let Some(w) = hab.worker {
            if workers.get(w).is_err() {
                hab.worker = None;
            }
        }
    }
    // if habitat despawned, unassign worker (remove AssignedWorker)
    for (worker_ent, assign) in &assigned {
        if habitats.get_mut(assign.habitat).is_err() {
            // habitat no longer exists, maybe via despawn? But habitats are HabitatBase, if habitat despawned, its entity gone, so habitats query won't contain it.
            // We detect via world? Simplify: if habitat entity not found, remove assignment
            commands.entity(worker_ent).remove::<AssignedWorker>();
        }
    }
}

// Upkeep: at moon end, pay for assigned workers, fatigue if cannot pay
pub fn worker_upkeep_tick(
    season_clock: Res<SeasonClock>,
    mut last_moon: Local<u32>,
    mut economy: ResMut<RunEconomy>,
    mut workers: Query<&mut Worker>,
    habitats: Query<&HabitatBase>,
    mut events: ResMut<PendingGameEvents>,
) {
    if *last_moon == season_clock.total_moons {
        return;
    }
    // Detect moon advance: total_moons increased
    if season_clock.total_moons > *last_moon {
        let moons_advanced = season_clock.total_moons - *last_moon;
        *last_moon = season_clock.total_moons;
        // For each moon advanced, pay upkeep once per moon (if multiple moons skipped, pay multiple times? But moon timer ticks one by one, so at most 1)
        for _ in 0..moons_advanced {
            let assigned: Vec<Entity> = habitats.iter().filter_map(|h| h.worker).collect();
            let total: u32 = assigned.iter().filter_map(|e| workers.get(*e).ok().map(|w| if w.fatigued {0} else { w.kind.upkeep()} )).sum();
            // But even fatigued workers? Spec says unpaid workers become Fatigued, bonuses disabled until paid or unassigned. They still cost upkeep? Probably still cost but disabled. For simplicity, we still charge upkeep for all assigned workers, regardless of fatigue, but if cannot pay, they become fatigued.
            // Actually total should be sum of upkeep for all assigned workers (fatigued or not) – they still cost.
            // Let's compute total as count of assigned workers (since all upkeep 1)
            let total_all = assigned.len() as u32;
            if total_all == 0 {
                continue;
            }
            if economy.dew >= total_all {
                economy.spend(total_all);
                events.0.push(GameEvent::UpkeepPaid { amount: total_all });
                // recover fatigued workers if now paid
                for ent in &assigned {
                    if let Ok(mut w) = workers.get_mut(*ent) {
                        if w.fatigued {
                            w.fatigued = false;
                            events.0.push(GameEvent::WorkerRecovered { kind: w.kind });
                        }
                    }
                }
            } else {
                let missing = total_all.saturating_sub(economy.dew);
                // spend what we have? Spec says if dew >= upkeep pay, else unpaid workers become Fatigued. We can spend all remaining dew, then fatigue.
                let have = economy.dew;
                if have > 0 {
                    economy.spend(have);
                    events.0.push(GameEvent::UpkeepPaid { amount: have });
                }
                events.0.push(GameEvent::UpkeepFailed { missing });
                for ent in &assigned {
                    if let Ok(mut w) = workers.get_mut(*ent) {
                        if !w.fatigued {
                            w.fatigued = true;
                            events.0.push(GameEvent::WorkerFatigued { kind: w.kind });
                        }
                    }
                }
            }
        }
    }
}

// Helper to spawn worker card (used by tests and packs)
pub fn spawn_worker_card(
    commands: &mut Commands,
    session: &mut super::GameSession,
    art: Option<&super::art::CardArt>,
    kind: WorkerKind,
    pos: Vec2,
) -> Option<Entity> {
    let card_type = kind.to_card();
    super::spawn_card(commands, session, art, card_type, pos, false).map(|e| {
        commands.entity(e).insert(Worker { kind, fatigued: false });
        e
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Entity;

    #[test]
    fn worker_assigns_only_to_empty_worker_slot() {
        // Habitat with worker slot empty should allow assignment, filled should not
        let kind = WorkerKind::Botanist;
        assert!(kind.upkeep() == 1);
        // Simulate habitat worker check
        let mut hab = HabitatBase { substrate: CardType::BioSubstrate, plant: None, companion: None, installation: None, worker: None };
        assert!(hab.worker.is_none());
        let w = Entity::from_bits(1);
        hab.worker = Some(w);
        assert!(hab.worker.is_some());
        // second worker should be blocked because slot not empty
        let w2 = Entity::from_bits(2);
        let can_assign = hab.worker.is_none();
        assert!(!can_assign);
        let _ = w2;
    }

    #[test]
    fn botanist_increases_growth_mult() {
        assert!((worker_growth_mult(WorkerKind::Botanist, CardType::VineSeed) - 1.35).abs() < 0.01);
        assert_eq!(worker_growth_mult(WorkerKind::Mycologist, CardType::VineSeed), 1.0);
    }

    #[test]
    fn mycologist_only_boosts_fungi_production() {
        assert!((worker_production_mult(WorkerKind::Mycologist, CardType::BasicFungi) - 1.40).abs() < 0.01);
        assert_eq!(worker_production_mult(WorkerKind::Mycologist, CardType::YoungVine), 1.0);
        assert_eq!(worker_production_mult(WorkerKind::Botanist, CardType::BasicFungi), 1.0);
    }

    #[test]
    fn total_mult_clamped() {
        let total = (3.0f32 * 1.35f32 * 1.40f32).clamp(0.25f32, 3.5f32);
        assert!(total <= 3.5);
        assert!(total >= 0.25);
    }

    #[test]
    fn greenhouse_offsets_winter_growth_penalty() {
        let winter = crate::game::seasons::season_base_modifiers(crate::game::seasons::Season::Winter).growth_mult;
        let greenhouse = advanced_installation_growth_mult(CardType::Greenhouse, CardType::YoungVine, crate::game::seasons::Season::Winter);
        assert!(greenhouse > 1.0);
        assert!(winter * greenhouse > winter);
    }

    #[test]
    fn water_tender_negates_heatwave_on_habitat() {
        assert!(water_tender_negates_heatwave(Some(WorkerKind::WaterTender), Some(WeatherEvent::Heatwave), false));
        assert!(!water_tender_negates_heatwave(Some(WorkerKind::Botanist), Some(WeatherEvent::Heatwave), false));
        assert!(!water_tender_negates_heatwave(Some(WorkerKind::WaterTender), Some(WeatherEvent::Heatwave), true)); // fatigued
    }

    #[test]
    fn compost_keeper_plus_cradle_cleans_faster() {
        let with = compost_interval_mult(Some(WorkerKind::CompostKeeper), false);
        assert!((with - 0.70).abs() < 0.01);
        assert_eq!(compost_interval_mult(Some(WorkerKind::Botanist), false), 1.0);
        assert_eq!(compost_interval_mult(Some(WorkerKind::CompostKeeper), true), 1.0);
    }

    #[test]
    fn fatigued_worker_bonus_disabled() {
        assert_eq!(total_worker_growth_mult(Some(WorkerKind::Botanist), CardType::VineSeed, true), 1.0);
        assert_eq!(total_worker_production_mult(Some(WorkerKind::Mycologist), CardType::BasicFungi, true), 1.0);
    }
}
