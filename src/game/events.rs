use bevy::prelude::*;

use super::{BlueprintId, CardType};
use super::seasons::{Season, WeatherEvent};

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
}

#[derive(Resource, Default)]
pub struct PendingGameEvents(pub Vec<GameEvent>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackId {
    SoilAndSpore,
    Pollinator,
    Symbiosis,
}
