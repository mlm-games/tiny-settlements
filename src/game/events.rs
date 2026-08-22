use bevy::prelude::*;

use super::{BlueprintId, CardType};
use super::seasons::{Season, WeatherEvent};
use super::workers::WorkerKind;

/// Domain events for discovery / commissions / stats.
/// Systems push events instead of directly mutating those subsystems.
#[derive(Clone, Copy, Debug)]
pub enum GameEvent {
    Spawned { card: CardType, entity: Entity },
    Crafted { result: CardType },
    Planted { card: CardType },
    Grew { from: CardType, to: CardType },
    Produced { producer: CardType, result: CardType },
    Pollinated,
    Hatched { card: CardType },
    CleanedToxin,
    Sold { card: CardType, value: u32 },
    PackOpened { pack: PackId },
    BiodiversityChanged { value: u32 },
    HabitatPlaced {
        substrate: CardType,
        col: i32,
        row: i32,
    },
    Stacked {
        card: CardType,
        layer: &'static str,
        base_substrate: CardType,
    },
    SynergyActivated {
        name: &'static str,
        dew_bonus: u32,
    },
    SynergyTick {
        dew: u32,
    },
    ProjectStarted {
        blueprint: BlueprintId,
    },
    ProjectCompleted {
        blueprint: BlueprintId,
        output: CardType,
    },
    InstallationInstalled {
        installation: CardType,
        habitat: Entity,
    },
    BlueprintUnlocked {
        blueprint: BlueprintId,
    },
    SeasonChanged {
        season: Season,
        year: u32,
    },
    WeatherStarted {
        weather: WeatherEvent,
    },
    WeatherEnded {
        weather: WeatherEvent,
    },
    BlightStruck {
        habitat: Option<Entity>,
    },
    HarvestGranted {
        dew: u32,
    },
    WorkerHired {
        kind: crate::game::workers::WorkerKind,
    },
    WorkerAssigned {
        kind: crate::game::workers::WorkerKind,
        habitat: Entity,
    },
    WorkerUnassigned {
        kind: crate::game::workers::WorkerKind,
    },
    WorkerFatigued {
        kind: crate::game::workers::WorkerKind,
    },
    WorkerRecovered {
        kind: crate::game::workers::WorkerKind,
    },
    UpkeepPaid {
        amount: u32,
    },
    UpkeepFailed {
        missing: u32,
    },
    AdvancedStructureUnlocked {
        blueprint: BlueprintId,
    },
    GardenStarted {
        garden: crate::game::campaign::GardenId,
        seed: u64,
    },
    ObjectiveCompleted {
        objective: super::objectives::ObjectiveId,
    },
    GardenCompleted {
        garden: crate::game::campaign::GardenId,
        stars: u8,
    },
}

#[derive(Resource, Default)]
pub struct PendingGameEvents(pub Vec<GameEvent>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackId {
    SoilAndSpore,
    Pollinator,
    Symbiosis,
    Specialist,
}
