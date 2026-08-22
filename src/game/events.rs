use bevy::prelude::*;

use super::CardType;

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
}

#[derive(Resource, Default)]
pub struct PendingGameEvents(pub Vec<GameEvent>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackId {
    SoilAndSpore,
    Pollinator,
    Symbiosis,
}
