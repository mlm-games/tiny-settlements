use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

use super::{CardType, PackId};

#[derive(Clone, Copy)]
pub struct PackEntry {
    pub card: CardType,
    pub weight: u32,
    pub max_owned: Option<u16>,
}

pub struct PackDefinition {
    pub id: PackId,
    pub name: &'static str,
    pub cost: u32,
    pub draws: u8,
    pub required_discoveries: u16,
    pub required_commissions: u16,
    pub entries: &'static [PackEntry],
}

#[derive(Resource, Default)]
pub struct PackPurchaseQueue(pub Vec<PackId>);

#[derive(Resource)]
pub struct RunRng(pub StdRng);

impl Default for RunRng {
    fn default() -> Self {
        Self(StdRng::seed_from_u64(0))
    }
}

impl RunRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }
}

// --- pack tables ---

pub const SOIL_ENTRIES: &[PackEntry] = &[
    PackEntry {
        card: CardType::BioSubstrate,
        weight: 25,
        max_owned: Some(4),
    },
    PackEntry {
        card: CardType::SporePod,
        weight: 25,
        max_owned: Some(4),
    },
    PackEntry {
        card: CardType::NutrientSlime,
        weight: 30,
        max_owned: Some(6),
    },
    PackEntry {
        card: CardType::ProcessedNutrients,
        weight: 15,
        max_owned: Some(5),
    },
    PackEntry {
        card: CardType::VineSeed,
        weight: 5,
        max_owned: Some(2),
    },
];

pub const POLLINATOR_ENTRIES: &[PackEntry] = &[
    PackEntry {
        card: CardType::VineSeed,
        weight: 30,
        max_owned: Some(4),
    },
    PackEntry {
        card: CardType::FlutterwingSpore,
        weight: 25,
        max_owned: Some(3),
    },
    PackEntry {
        card: CardType::NutrientSlime,
        weight: 25,
        max_owned: Some(6),
    },
    PackEntry {
        card: CardType::ProcessedNutrients,
        weight: 15,
        max_owned: Some(5),
    },
    PackEntry {
        card: CardType::FertilizedVinePod,
        weight: 5,
        max_owned: Some(2),
    },
];

pub const SYMBIOSIS_ENTRIES: &[PackEntry] = &[
    PackEntry {
        card: CardType::GrazingSlugEgg,
        weight: 25,
        max_owned: Some(3),
    },
    PackEntry {
        card: CardType::FertilizedVinePod,
        weight: 25,
        max_owned: Some(3),
    },
    PackEntry {
        card: CardType::RichMulch,
        weight: 20,
        max_owned: Some(4),
    },
    PackEntry {
        card: CardType::LuminaCrystal,
        weight: 20,
        max_owned: Some(3),
    },
    PackEntry {
        card: CardType::ApexSpore,
        weight: 10,
        max_owned: Some(1),
    },
];

pub const SPECIALIST_ENTRIES: &[PackEntry] = &[
    PackEntry {
        card: CardType::Botanist,
        weight: 18,
        max_owned: Some(2),
    },
    PackEntry {
        card: CardType::Mycologist,
        weight: 18,
        max_owned: Some(2),
    },
    PackEntry {
        card: CardType::Entomologist,
        weight: 18,
        max_owned: Some(2),
    },
    PackEntry {
        card: CardType::CompostKeeper,
        weight: 18,
        max_owned: Some(2),
    },
    PackEntry {
        card: CardType::WaterTender,
        weight: 18,
        max_owned: Some(2),
    },
    PackEntry {
        card: CardType::NutrientSlime,
        weight: 10,
        max_owned: Some(6),
    },
];

pub const PACKS: &[PackDefinition] = &[
    PackDefinition {
        id: PackId::SoilAndSpore,
        name: "Soil & Spore",
        cost: 4,
        draws: 2,
        required_discoveries: 0,
        required_commissions: 0,
        entries: SOIL_ENTRIES,
    },
    PackDefinition {
        id: PackId::Pollinator,
        name: "Pollinator",
        cost: 9,
        draws: 2,
        required_discoveries: 5,
        required_commissions: 0,
        entries: POLLINATOR_ENTRIES,
    },
    PackDefinition {
        id: PackId::Symbiosis,
        name: "Symbiosis",
        cost: 15,
        draws: 3,
        required_discoveries: 10,
        required_commissions: 3,
        entries: SYMBIOSIS_ENTRIES,
    },
    PackDefinition {
        id: PackId::Specialist,
        name: "Specialist",
        cost: 14,
        draws: 2,
        required_discoveries: 8,
        required_commissions: 2,
        entries: SPECIALIST_ENTRIES,
    },
];

pub fn pack_definition(id: PackId) -> &'static PackDefinition {
    PACKS.iter().find(|p| p.id == id).expect("unknown pack")
}

pub fn pack_id_from_str(s: &str) -> Option<PackId> {
    match s {
        "soil_and_spore" => Some(PackId::SoilAndSpore),
        "pollinator" => Some(PackId::Pollinator),
        "symbiosis" => Some(PackId::Symbiosis),
        "specialist" => Some(PackId::Specialist),
        _ => None,
    }
}

pub fn pack_id_to_str(id: PackId) -> &'static str {
    match id {
        PackId::SoilAndSpore => "soil_and_spore",
        PackId::Pollinator => "pollinator",
        PackId::Symbiosis => "symbiosis",
        PackId::Specialist => "specialist",
    }
}

pub fn is_pack_unlocked(
    def: &PackDefinition,
    discoveries: u16,
    commissions_completed: u16,
) -> bool {
    discoveries >= def.required_discoveries && commissions_completed >= def.required_commissions
}

/// Filter entries whose live count >= max_owned.
pub fn available_entries<'a>(
    entries: &'a [PackEntry],
    live_counts: &dyn Fn(CardType) -> u32,
) -> Vec<&'a PackEntry> {
    entries
        .iter()
        .filter(|e| {
            if let Some(max) = e.max_owned {
                live_counts(e.card) < max as u32
            } else {
                true
            }
        })
        .collect()
}

/// Perform `draws` weighted draws from `entries`, filtering by max_owned.
/// Uses the provided seeded rng. Returns None if no entry available at draw time.
/// Falls back to allow draws even if filtered empty? No — return empty to avoid deadlock;
/// caller should handle empty gracefully.
pub fn weighted_draws(
    rng: &mut StdRng,
    entries: &[PackEntry],
    draws: u8,
    live_counts: &dyn Fn(CardType) -> u32,
) -> Vec<CardType> {
    let mut out = Vec::new();
    for _ in 0..draws {
        let pool = available_entries(entries, live_counts);
        if pool.is_empty() {
            break;
        }
        let total: u32 = pool.iter().map(|e| e.weight).sum();
        if total == 0 {
            break;
        }
        let mut roll = rng.random_range(0..total);
        let mut chosen = pool[0].card;
        for e in &pool {
            if roll < e.weight {
                chosen = e.card;
                break;
            }
            roll -= e.weight;
        }
        out.push(chosen);
    }
    out
}

/// Convenience: draw for a pack definition.
pub fn draw_for_pack(
    rng: &mut StdRng,
    def: &PackDefinition,
    live_counts: &dyn Fn(CardType) -> u32,
) -> Vec<CardType> {
    weighted_draws(rng, def.entries, def.draws, live_counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts_stub(map: &std::collections::HashMap<CardType, u32>) -> impl Fn(CardType) -> u32 + '_ {
        move |c| map.get(&c).copied().unwrap_or(0)
    }

    #[test]
    fn draw_deterministic_with_seed() {
        let mut a = StdRng::seed_from_u64(42);
        let mut b = StdRng::seed_from_u64(42);
        let def = pack_definition(PackId::SoilAndSpore);
        let empty: std::collections::HashMap<CardType, u32> = std::collections::HashMap::new();
        let da = draw_for_pack(&mut a, def, &counts_stub(&empty));
        let db = draw_for_pack(&mut b, def, &counts_stub(&empty));
        assert_eq!(da, db);
        assert_eq!(da.len(), 2);
    }

    #[test]
    fn max_owned_filters() {
        let mut map = std::collections::HashMap::new();
        map.insert(CardType::BioSubstrate, 4);
        map.insert(CardType::SporePod, 4);
        map.insert(CardType::NutrientSlime, 6);
        map.insert(CardType::ProcessedNutrients, 5);
        // only VineSeed remains
        let mut rng = StdRng::seed_from_u64(1);
        let def = pack_definition(PackId::SoilAndSpore);
        let draws = draw_for_pack(&mut rng, def, &counts_stub(&map));
        assert!(draws.iter().all(|c| *c == CardType::VineSeed));
    }

    #[test]
    fn unlock_gating() {
        let soil = pack_definition(PackId::SoilAndSpore);
        assert!(is_pack_unlocked(soil, 0, 0));
        let poll = pack_definition(PackId::Pollinator);
        assert!(!is_pack_unlocked(poll, 4, 0));
        assert!(is_pack_unlocked(poll, 5, 0));
        let sym = pack_definition(PackId::Symbiosis);
        assert!(!is_pack_unlocked(sym, 10, 2));
        assert!(is_pack_unlocked(sym, 10, 3));
    }

    #[test]
    fn no_deadlock_when_all_maxed() {
        let mut map = std::collections::HashMap::new();
        for e in SOIL_ENTRIES {
            if let Some(m) = e.max_owned {
                map.insert(e.card, m as u32);
            }
        }
        let mut rng = StdRng::seed_from_u64(99);
        let def = pack_definition(PackId::SoilAndSpore);
        let draws = draw_for_pack(&mut rng, def, &counts_stub(&map));
        assert!(draws.is_empty());
    }
}
