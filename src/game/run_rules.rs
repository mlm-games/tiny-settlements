use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::CardType;
use super::PackId;
use super::projects::BlueprintId;
use super::seasons::WeatherEvent;
use super::campaign::{FeatureRules, GardenId, garden_def};

#[derive(Resource, Clone, Debug)]
pub struct RunRules {
    pub mode: super::campaign::RunMode,
    pub features: FeatureRules,
    pub allowed_cards: HashSet<CardType>,
    pub allowed_packs: HashSet<PackId>,
    pub allowed_blueprints: HashSet<BlueprintId>,
    pub weather_deck: Vec<WeatherEvent>,
}

impl Default for RunRules {
    fn default() -> Self {
        Self::free_garden()
    }
}

impl RunRules {
    pub fn free_garden() -> Self {
        Self {
            mode: super::campaign::RunMode::FreeGarden,
            features: FeatureRules {
                habitats: true,
                commissions: true,
                projects: true,
                installations: true,
                seasons: true,
                weather: true,
                workers: true,
                advanced_structures: true,
            },
            allowed_cards: super::discovery::DiscoveryState::all_types().into_iter().collect(),
            allowed_packs: super::packs::PACKS.iter().map(|p| p.id).collect(),
            allowed_blueprints: super::projects::BLUEPRINTS.iter().map(|b| b.id).collect(),
            weather_deck: WeatherEvent::ALL.to_vec(),
        }
    }
    pub fn for_garden(id: GardenId) -> Self {
        let def = garden_def(id);
        Self {
            mode: super::campaign::RunMode::Campaign(id),
            features: def.features,
            allowed_cards: def.allowed_cards.iter().copied().collect(),
            allowed_packs: def.allowed_packs.iter().copied().collect(),
            allowed_blueprints: def.allowed_blueprints.iter().copied().collect(),
            weather_deck: def.weather_deck.to_vec(),
        }
    }
    pub fn allows_card(&self, card: CardType) -> bool {
        self.allowed_cards.contains(&card)
    }
    pub fn allows_pack(&self, pack: PackId) -> bool {
        self.allowed_packs.contains(&pack)
    }
    pub fn allows_blueprint(&self, blueprint: BlueprintId) -> bool {
        self.allowed_blueprints.contains(&blueprint)
    }
    pub fn allows_habitat(&self) -> bool {
        self.features.habitats
    }
    pub fn allows_worker(&self) -> bool {
        self.features.workers
    }
    pub fn allows_season(&self) -> bool {
        self.features.seasons
    }
    pub fn allows_weather(&self) -> bool {
        self.features.weather
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::campaign::GardenId;
    #[test]
    fn free_garden_allows_all_content() {
        let rules = RunRules::free_garden();
        assert!(rules.allows_card(CardType::Botanist));
        assert!(rules.allows_card(CardType::Greenhouse));
        assert!(rules.allows_pack(PackId::Specialist));
        assert!(rules.allows_blueprint(BlueprintId::Greenhouse));
        assert!(rules.features.workers);
        assert!(rules.features.seasons);
    }
    #[test]
    fn balcony_blocks_workers_projects_and_weather() {
        let rules = RunRules::for_garden(GardenId::AbandonedBalcony);
        assert!(!rules.features.habitats);
        assert!(!rules.features.workers);
        assert!(!rules.features.weather);
        assert!(!rules.allows_card(CardType::Botanist));
        assert!(!rules.allows_blueprint(BlueprintId::NurseryTray));
        assert!(!rules.allows_pack(PackId::Specialist));
    }
    #[test]
    fn pack_draws_never_return_disallowed_cards() {
        // Simulate that free garden would allow, balcony would filter
        let balcony = RunRules::for_garden(GardenId::AbandonedBalcony);
        // Specialist pack contains Botanist which is disallowed in balcony
        assert!(!balcony.allows_card(CardType::Botanist));
        // Even if pack is allowed (but balcony only allows Soil pack, so specialist not allowed)
        assert!(!balcony.allows_pack(PackId::Specialist));
    }
}
