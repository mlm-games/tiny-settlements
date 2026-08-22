//! Habitat stacks + spatial synergy (Phase 2).
//!
//! Substrate cards snap to a 6×3 grid and become HabitatBase.
//! Plants and companions stack onto habitats with visual offsets.
//! Neighbor diversity boosts production; monoculture is flagged.

use bevy::prelude::*;

use super::card_defs::CardType;
use super::events::{GameEvent, PendingGameEvents};
use super::{Card, GameCleanup, GameSession, CARD_SIZE};


pub const GRID_ORIGIN: Vec2 = Vec2::new(-330.0, -160.0);
pub const GRID_CELL: Vec2 = Vec2::new(108.0, 140.0);
pub const MAX_COLS: i32 = 6;
pub const MAX_ROWS: i32 = 3;
pub const STACK_SNAP_DIST: f32 = 78.0;

pub fn grid_to_world(col: i32, row: i32) -> Vec2 {
    GRID_ORIGIN + Vec2::new(col as f32 * GRID_CELL.x, row as f32 * GRID_CELL.y)
}

pub fn world_to_grid(pos: Vec2) -> Option<(i32, i32)> {
    let rel = pos - GRID_ORIGIN + GRID_CELL * 0.5;
    let col = (rel.x / GRID_CELL.x).floor() as i32;
    let row = (rel.y / GRID_CELL.y).floor() as i32;
    if (0..MAX_COLS).contains(&col) && (0..MAX_ROWS).contains(&row) {
        Some((col, row))
    } else {
        None
    }
}


#[derive(Component)]
pub struct HabitatBase {
    pub substrate: CardType,
    pub plant: Option<Entity>,
    pub companion: Option<Entity>,
    pub installation: Option<Entity>,
    pub worker: Option<Entity>,
}

#[derive(Component)]
pub struct StackedOn {
    pub base: Entity,
    pub layer: StackLayer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackLayer {
    Plant,
    Companion,
    Installation,
}

#[derive(Component, Default, Clone)]
pub struct HabitatSynergy {
    pub neighbor_plant_types: Vec<CardType>,
    pub diversity: u32,
    pub production_mult: f32,
    pub is_monoculture: bool,
    pub active_combo: Option<&'static str>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct GridSlot {
    pub col: i32,
    pub row: i32,
}

/// Visual marker for empty grid cells (optional debug / ghost).
#[derive(Component)]
pub struct GridGhost;


pub fn is_habitat_substrate(t: CardType) -> bool {
    matches!(t, CardType::BioSubstrate | CardType::FertileSubstrate)
}

pub fn can_stack_as_plant(t: CardType) -> bool {
    matches!(
        t,
        CardType::BasicFungi
            | CardType::YoungVine
            | CardType::MatureVine
            | CardType::SymbioticAlgae
            | CardType::GrowingApex
            | CardType::GenesisBloom
    )
}

pub fn can_stack_as_companion(t: CardType) -> bool {
    matches!(
        t,
        CardType::MatureFlutterwing | CardType::GrazingSlug | CardType::FlutterwingLarva
    )
}

pub fn can_stack_as_installation(t: CardType) -> bool {
    t.is_installation()
}

pub fn substrate_growth_mult(t: CardType) -> f32 {
    match t {
        CardType::FertileSubstrate => 1.35,
        CardType::BioSubstrate => 1.0,
        _ => 1.0,
    }
}


#[derive(Clone, Copy, Debug)]
pub struct SynergyCombo {
    pub name: &'static str,
    pub plant: CardType,
    pub companion: CardType,
    pub production_bonus: f32,
    pub dew_per_tick: u32,
}

pub const SYNERGY_COMBOS: &[SynergyCombo] = &[
    SynergyCombo {
        name: "Pollinator's Garden",
        plant: CardType::MatureVine,
        companion: CardType::MatureFlutterwing,
        production_bonus: 0.50,
        dew_per_tick: 2,
    },
    SynergyCombo {
        name: "Fungal Grazing",
        plant: CardType::BasicFungi,
        companion: CardType::GrazingSlug,
        production_bonus: 0.40,
        dew_per_tick: 1,
    },
    SynergyCombo {
        name: "Algae Nursery",
        plant: CardType::SymbioticAlgae,
        companion: CardType::FlutterwingLarva,
        production_bonus: 0.60,
        dew_per_tick: 3,
    },
];

pub fn find_synergy(plant: CardType, companion: CardType) -> Option<&'static SynergyCombo> {
    SYNERGY_COMBOS
        .iter()
        .find(|s| s.plant == plant && s.companion == companion)
}

//    We avoid depending on private PendingFx by pushing via Events resource.
//    For FX, we use a simple helper that spawns via PendingFx if available,
//    otherwise we just emit GameEvent and let mod.rs handle FX.
//    To keep spec compatibility, we provide helpers that take generic callbacks.


/// Drop substrate onto a free grid cell → become HabitatBase.
/// Returns snap position if placed, None otherwise.
/// Caller is responsible for inserting components and emitting events/FX.
pub fn try_place_habitat(
    entity: Entity,
    card_type: CardType,
    drop_pos: Vec2,
    occupied: &Query<&GridSlot, With<HabitatBase>>,
) -> Option<(i32, i32, Vec2)> {
    if !is_habitat_substrate(card_type) {
        return None;
    }
    let (col, row) = world_to_grid(drop_pos)?;
    for slot in occupied.iter() {
        if slot.col == col && slot.row == row {
            return None;
        }
    }
    let snap = grid_to_world(col, row);
    Some((col, row, snap))
}

/// Drop plant/companion/installation onto a nearby habitat.
/// Returns Some((base_entity, layer)) if stack possible.
pub fn find_stack_target(
    card_type: CardType,
    drop_pos: Vec2,
    habitats: &Query<(Entity, &GridSlot, &HabitatBase)>,
) -> Option<(Entity, StackLayer)> {
    // Prefer nearest habitat within snap distance
    let mut best: Option<(Entity, f32, StackLayer)> = None;
    for (e, slot, hab) in habitats.iter() {
        let pos = grid_to_world(slot.col, slot.row);
        let d = drop_pos.distance(pos);
        if d <= STACK_SNAP_DIST {
            let needed_layer = if can_stack_as_plant(card_type) && hab.plant.is_none() {
                Some(StackLayer::Plant)
            } else if can_stack_as_companion(card_type)
                && hab.plant.is_some()
                && hab.companion.is_none()
            {
                Some(StackLayer::Companion)
            } else if can_stack_as_installation(card_type) && hab.installation.is_none() {
                Some(StackLayer::Installation)
            } else {
                None
            };
            if let Some(layer) = needed_layer {
                if best.map_or(true, |(_, bd, _)| d < bd) {
                    best = Some((e, d, layer));
                }
            }
        }
    }
    best.map(|(e, _, layer)| (e, layer))
}

/// Find installation-specific target (for explicit installation stacking).
pub fn find_installation_target(
    card_type: CardType,
    drop_pos: Vec2,
    habitats: &Query<(Entity, &GridSlot, &HabitatBase)>,
) -> Option<Entity> {
    if !can_stack_as_installation(card_type) {
        return None;
    }
    let mut best: Option<(Entity, f32)> = None;
    for (e, slot, hab) in habitats.iter() {
        if hab.installation.is_some() {
            continue;
        }
        let pos = grid_to_world(slot.col, slot.row);
        let d = drop_pos.distance(pos);
        if d <= STACK_SNAP_DIST {
            if best.map_or(true, |(_, bd)| d < bd) {
                best = Some((e, d));
            }
        }
    }
    best.map(|(e, _)| e)
}


pub fn spawn_grid_ghosts(mut commands: Commands) {
    for col in 0..MAX_COLS {
        for row in 0..MAX_ROWS {
            let pos = grid_to_world(col, row);
            commands.spawn((
                GameCleanup,
                GridGhost,
                Sprite {
                    color: Color::srgba(0.25, 0.40, 0.28, 0.18),
                    custom_size: Some(Vec2::new(CARD_SIZE.x * 0.92, CARD_SIZE.y * 0.55)),
                    ..default()
                },
                Transform::from_translation(pos.extend(-8.0)),
            ));
        }
    }
}

pub fn recompute_synergies(
    mut habitats: Query<(Entity, &GridSlot, &HabitatBase, &mut HabitatSynergy)>,
    cards: Query<&Card>,
) {
    let snap: Vec<(Entity, i32, i32, Option<CardType>, Option<CardType>)> = habitats
        .iter()
        .map(|(e, slot, hab, _)| {
            let plant = hab
                .plant
                .and_then(|p| cards.get(p).ok())
                .map(|c| c.card_type);
            let companion = hab
                .companion
                .and_then(|p| cards.get(p).ok())
                .map(|c| c.card_type);
            (e, slot.col, slot.row, plant, companion)
        })
        .collect();

    for (e, col, row, plant, companion) in &snap {
        let mut neighbor_plants = Vec::new();
        for (_, nc, nr, np, _) in &snap {
            if nc == col && nr == row {
                continue;
            }
            if (nc - col).abs() <= 1 && (nr - row).abs() <= 1 {
                if let Some(t) = np {
                    neighbor_plants.push(*t);
                }
            }
        }

        let mut unique = neighbor_plants.clone();
        unique.sort_by_key(|t| *t as u32);
        unique.dedup();
        let diversity = unique.len() as u32;

        let is_mono = neighbor_plants.len() >= 2
            && neighbor_plants.iter().all(|t| *t == neighbor_plants[0]);

        let mut mult = 1.0 + diversity as f32 * 0.12;
        if is_mono {
            mult = 1.25; // efficient but risky
        }

        let mut combo_name = None;
        if let (Some(p), Some(c)) = (plant, companion) {
            if let Some(combo) = find_synergy(*p, *c) {
                mult += combo.production_bonus;
                combo_name = Some(combo.name);
            }
        }

        // Need substrate for final mult; fetch hab
        if let Ok((_, _, hab, mut syn)) = habitats.get_mut(*e) {
            mult *= substrate_growth_mult(hab.substrate);
            syn.neighbor_plant_types = neighbor_plants;
            syn.diversity = diversity;
            syn.production_mult = mult;
            syn.is_monoculture = is_mono;
            syn.active_combo = combo_name;
        }
    }
    // Ensure habitats with no plant still have default mult computed correctly
    // (already handled above, initial mult = 1.0 + diversity*0.12)
}

pub fn position_stacked_cards(
    bases: Query<(&Transform, &HabitatBase), Without<StackedOn>>,
    mut stacked: Query<(&StackedOn, &mut Transform)>,
) {
    for (stack, mut tf) in &mut stacked {
        let Ok((base_tf, _)) = bases.get(stack.base) else {
            continue;
        };
        let b = base_tf.translation;
        match stack.layer {
            StackLayer::Plant => {
                tf.translation = Vec3::new(b.x, b.y + 20.0, b.z + 3.0);
            }
            StackLayer::Companion => {
                tf.translation = Vec3::new(b.x + 14.0, b.y + 40.0, b.z + 5.0);
            }
            StackLayer::Installation => {
                tf.translation = Vec3::new(b.x - 18.0, b.y + 58.0, b.z + 7.0);
            }
        }
    }
}

/// Clear habitat slots when stacked cards despawn; drop companion if plant dies.
pub fn clear_dead_stacks(
    mut commands: Commands,
    mut habitats: Query<&mut HabitatBase>,
    cards: Query<Entity, With<Card>>,
) {
    for mut hab in &mut habitats {
        if let Some(p) = hab.plant {
            if cards.get(p).is_err() {
                hab.plant = None;
                if let Some(c) = hab.companion.take() {
                    commands.entity(c).remove::<StackedOn>();
                }
            }
        }
        if let Some(c) = hab.companion {
            if cards.get(c).is_err() {
                hab.companion = None;
            }
        }
        if let Some(i) = hab.installation {
            if cards.get(i).is_err() {
                hab.installation = None;
            }
        }
    }
}

/// Periodic Dew from diverse / combo habitats.
pub fn synergy_income_tick(
    time: Res<Time>,
    mut timer: Local<Timer>,
    session: Res<GameSession>,
    habitats: Query<(&HabitatBase, &HabitatSynergy)>,
    cards: Query<&Card>,
    mut economy: ResMut<super::economy::RunEconomy>,
    mut events: ResMut<PendingGameEvents>,
    mut pending_fx: ResMut<super::PendingFx>,
) {
    if session.game_over {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(10.0, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    let mut dew = 0u32;
    for (hab, syn) in &habitats {
        if hab.plant.is_none() {
            continue;
        }
        if syn.diversity >= 2 {
            dew += syn.diversity; // 2+ unique neighbors
        }
        if let (Some(p), Some(c)) = (hab.plant, hab.companion) {
            let pt = cards.get(p).map(|x| x.card_type).ok();
            let ct = cards.get(c).map(|x| x.card_type).ok();
            if let (Some(pt), Some(ct)) = (pt, ct) {
                if let Some(combo) = find_synergy(pt, ct) {
                    dew += combo.dew_per_tick;
                }
            }
        }
        // Infrastructure: Dew Basin and Pollinator Lodge bonuses
        if let Some(inst) = hab.installation {
            if let Ok(ic) = cards.get(inst) {
                match ic.card_type {
                    CardType::DewBasin => {
                        // Only counts if habitat has plant (we already checked plant is_some)
                        dew += 1;
                    }
                    CardType::PollinatorLodge => {
                        if let Some(comp) = hab.companion {
                            if let Ok(cc) = cards.get(comp) {
                                if cc.card_type == CardType::MatureFlutterwing {
                                    dew += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if dew > 0 {
        economy.earn(dew);
        events.0.push(GameEvent::SynergyTick { dew });
        pending_fx.0.push(super::FxEvent::Produce {
            pos: Vec2::ZERO,
            color: Color::srgb(0.45, 0.85, 0.55),
        });
    }
}

/// Production mult for a producer entity if stacked on a habitat.
pub fn production_mult_for(
    entity: Entity,
    stacked: &Query<&StackedOn>,
    habitats: &Query<&HabitatSynergy>,
) -> f32 {
    stacked
        .get(entity)
        .ok()
        .and_then(|s| habitats.get(s.base).ok())
        .map(|syn| syn.production_mult)
        .unwrap_or(1.0)
}

// Alternate overload for habitats query via HabitatBase+Synergy combo
pub fn production_mult_for_entity(
    entity: Entity,
    stacked: &Query<&StackedOn>,
    habitats_syn: &Query<&HabitatSynergy, With<HabitatBase>>,
) -> f32 {
    stacked
        .get(entity)
        .ok()
        .and_then(|s| habitats_syn.get(s.base).ok())
        .map(|syn| syn.production_mult)
        .unwrap_or(1.0)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_round_trip() {
        for col in 0..MAX_COLS {
            for row in 0..MAX_ROWS {
                let w = grid_to_world(col, row);
                assert_eq!(world_to_grid(w), Some((col, row)));
            }
        }
    }

    #[test]
    fn oob_none() {
        assert!(world_to_grid(Vec2::splat(-9999.0)).is_none());
    }

    #[test]
    fn stacking_rules() {
        assert!(is_habitat_substrate(CardType::BioSubstrate));
        assert!(can_stack_as_plant(CardType::BasicFungi));
        assert!(can_stack_as_companion(CardType::MatureFlutterwing));
        assert!(!can_stack_as_plant(CardType::Gardener));
    }

    #[test]
    fn synergy_lookup() {
        assert!(find_synergy(CardType::MatureVine, CardType::MatureFlutterwing).is_some());
        assert!(find_synergy(CardType::BasicFungi, CardType::Gardener).is_none());
    }

    #[test]
    fn fertile_faster() {
        assert!(substrate_growth_mult(CardType::FertileSubstrate) > 1.0);
    }
}
