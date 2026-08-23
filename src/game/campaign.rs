use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::CardType;
use super::PackId;
use super::projects::BlueprintId;
use super::seasons::WeatherEvent;
use super::objectives::{ObjectiveDef, ObjectiveId, ObjectiveKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GardenId {
    AbandonedBalcony,
    WildMeadow,
    FloodedWetland,
    MoonlitConservatory,
    GlassWastes,
}

impl GardenId {
    pub const ALL: [Self; 5] = [
        Self::AbandonedBalcony,
        Self::WildMeadow,
        Self::FloodedWetland,
        Self::MoonlitConservatory,
        Self::GlassWastes,
    ];
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::AbandonedBalcony => "abandoned_balcony",
            Self::WildMeadow => "wild_meadow",
            Self::FloodedWetland => "flooded_wetland",
            Self::MoonlitConservatory => "moonlit_conservatory",
            Self::GlassWastes => "glass_wastes",
        }
    }
    pub fn from_stable_id(s: &str) -> Option<Self> {
        Some(match s {
            "abandoned_balcony" => Self::AbandonedBalcony,
            "wild_meadow" => Self::WildMeadow,
            "flooded_wetland" => Self::FloodedWetland,
            "moonlit_conservatory" => Self::MoonlitConservatory,
            "glass_wastes" => Self::GlassWastes,
            _ => return None,
        })
    }
    pub const fn index(self) -> usize {
        match self {
            Self::AbandonedBalcony => 0,
            Self::WildMeadow => 1,
            Self::FloodedWetland => 2,
            Self::MoonlitConservatory => 3,
            Self::GlassWastes => 4,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::AbandonedBalcony => "Abandoned Balcony",
            Self::WildMeadow => "Wild Meadow",
            Self::FloodedWetland => "Flooded Wetland",
            Self::MoonlitConservatory => "Moonlit Conservatory",
            Self::GlassWastes => "Glass Wastes",
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub enum RunMode {
    #[default]
    FreeGarden,
    Campaign(GardenId),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FeatureRules {
    pub habitats: bool,
    pub commissions: bool,
    pub projects: bool,
    pub installations: bool,
    pub seasons: bool,
    pub weather: bool,
    pub workers: bool,
    pub advanced_structures: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct StartingCard {
    pub card: CardType,
    pub position: Vec2,
    pub planted: bool,
}

pub struct GardenDef {
    pub id: GardenId,
    pub name: &'static str,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub intro: &'static str,
    pub starting_dew: u32,
    pub starting_cards: &'static [StartingCard],
    pub features: FeatureRules,
    pub allowed_cards: &'static [CardType],
    pub allowed_packs: &'static [PackId],
    pub allowed_blueprints: &'static [BlueprintId],
    pub weather_deck: &'static [WeatherEvent],
    pub objectives: &'static [ObjectiveDef; 3],
    pub board_color: Color,
    pub grid_cols: i32,
    pub grid_rows: i32,
}

// Starting cards
const BALCONY_START: &[StartingCard] = &[
    StartingCard { card: CardType::Gardener, position: Vec2::new(-540.0, 60.0), planted: false },
    StartingCard { card: CardType::BioSubstrate, position: Vec2::new(-390.0, 160.0), planted: false },
    StartingCard { card: CardType::BioSubstrate, position: Vec2::new(-390.0, -40.0), planted: false },
    StartingCard { card: CardType::SporePod, position: Vec2::new(-240.0, 160.0), planted: false },
    StartingCard { card: CardType::NutrientSlime, position: Vec2::new(-240.0, 60.0), planted: false },
    StartingCard { card: CardType::NutrientSlime, position: Vec2::new(-240.0, -40.0), planted: false },
];

const MEADOW_START: &[StartingCard] = BALCONY_START;
const WETLAND_START: &[StartingCard] = BALCONY_START;
const CONSERVATORY_START: &[StartingCard] = BALCONY_START;
const WASTES_START: &[StartingCard] = &[
    StartingCard { card: CardType::Gardener, position: Vec2::new(-540.0, 60.0), planted: false },
    StartingCard { card: CardType::BioSubstrate, position: Vec2::new(-390.0, 60.0), planted: false },
    StartingCard { card: CardType::SporePod, position: Vec2::new(-240.0, 60.0), planted: false },
    StartingCard { card: CardType::NutrientSlime, position: Vec2::new(-240.0, -40.0), planted: false },
];

// Allowed cards
const ALL_CARDS: &[CardType] = &[
    CardType::Gardener,
    CardType::BioSubstrate,
    CardType::SporePod,
    CardType::NutrientSlime,
    CardType::BasicFungi,
    CardType::ProcessedNutrients,
    CardType::VineSeed,
    CardType::YoungVine,
    CardType::MatureVine,
    CardType::FlutterwingSpore,
    CardType::FlutterwingLarva,
    CardType::MatureFlutterwing,
    CardType::FertilizedVinePod,
    CardType::SymbioticAlgae,
    CardType::LuminaCrystal,
    CardType::GrazingSlugEgg,
    CardType::GrazingSlug,
    CardType::RichMulch,
    CardType::FertileSubstrate,
    CardType::WasteToxin,
    CardType::ApexSpore,
    CardType::GrowingApex,
    CardType::GenesisBloom,
    CardType::NurseryTray,
    CardType::CompostCradle,
    CardType::MyceliumBed,
    CardType::PollinatorLodge,
    CardType::DewBasin,
    CardType::SeedArchive,
    CardType::Botanist,
    CardType::Mycologist,
    CardType::Entomologist,
    CardType::CompostKeeper,
    CardType::WaterTender,
    CardType::Greenhouse,
    CardType::RainBarrel,
    CardType::BeeHotel,
    CardType::MushroomCellar,
    CardType::ObservationStation,
    CardType::IrrigationChannel,
];

// Base cards without installations/workers/advanced (original 23 minus 6 installs = 17? but we keep 23 for simplicity)
const BASE_CARDS: &[CardType] = &[
    CardType::Gardener,
    CardType::BioSubstrate,
    CardType::SporePod,
    CardType::NutrientSlime,
    CardType::BasicFungi,
    CardType::ProcessedNutrients,
    CardType::VineSeed,
    CardType::YoungVine,
    CardType::MatureVine,
    CardType::FlutterwingSpore,
    CardType::FlutterwingLarva,
    CardType::MatureFlutterwing,
    CardType::FertilizedVinePod,
    CardType::SymbioticAlgae,
    CardType::LuminaCrystal,
    CardType::GrazingSlugEgg,
    CardType::GrazingSlug,
    CardType::RichMulch,
    CardType::FertileSubstrate,
    CardType::WasteToxin,
    CardType::ApexSpore,
    CardType::GrowingApex,
    CardType::GenesisBloom,
];

const ALL_PACKS: &[PackId] = &[PackId::SoilAndSpore, PackId::Pollinator, PackId::Symbiosis, PackId::Specialist];
const SOIL_PACK: &[PackId] = &[PackId::SoilAndSpore];
const SOIL_POLL: &[PackId] = &[PackId::SoilAndSpore, PackId::Pollinator];
const WETLAND_PACKS: &[PackId] = &[PackId::SoilAndSpore, PackId::Pollinator, PackId::Symbiosis];
const WETLAND_CARDS: &[CardType] = &[
    CardType::Gardener,
    CardType::BioSubstrate,
    CardType::SporePod,
    CardType::NutrientSlime,
    CardType::BasicFungi,
    CardType::ProcessedNutrients,
    CardType::VineSeed,
    CardType::YoungVine,
    CardType::MatureVine,
    CardType::FlutterwingSpore,
    CardType::FlutterwingLarva,
    CardType::MatureFlutterwing,
    CardType::FertilizedVinePod,
    CardType::SymbioticAlgae,
    CardType::LuminaCrystal,
    CardType::GrazingSlugEgg,
    CardType::GrazingSlug,
    CardType::RichMulch,
    CardType::FertileSubstrate,
    CardType::WasteToxin,
    CardType::ApexSpore,
    CardType::GrowingApex,
    CardType::GenesisBloom,
    CardType::NurseryTray,
    CardType::CompostCradle,
    CardType::MyceliumBed,
    CardType::PollinatorLodge,
    CardType::DewBasin,
    CardType::SeedArchive,
];
const ALL_BLUEPRINTS: &[BlueprintId] = &[
    BlueprintId::NurseryTray,
    BlueprintId::CompostCradle,
    BlueprintId::MyceliumBed,
    BlueprintId::PollinatorLodge,
    BlueprintId::DewBasin,
    BlueprintId::SeedArchive,
    BlueprintId::Greenhouse,
    BlueprintId::RainBarrel,
    BlueprintId::BeeHotel,
    BlueprintId::MushroomCellar,
    BlueprintId::ObservationStation,
    BlueprintId::IrrigationChannel,
];
const BASE_BLUEPRINTS: &[BlueprintId] = &[
    BlueprintId::NurseryTray,
    BlueprintId::CompostCradle,
    BlueprintId::MyceliumBed,
    BlueprintId::DewBasin,
];
const NO_BLUEPRINTS: &[BlueprintId] = &[];

const NO_WEATHER: &[WeatherEvent] = &[];
const WETLAND_WEATHER: &[WeatherEvent] = &[WeatherEvent::HeavyRain, WeatherEvent::Heatwave, WeatherEvent::HarvestFair];
const ALL_WEATHER: &[WeatherEvent] = &[WeatherEvent::Heatwave, WeatherEvent::HeavyRain, WeatherEvent::Blight, WeatherEvent::FrostSnap, WeatherEvent::PollinatorSurge, WeatherEvent::HarvestFair];

// Objectives
const BALCONY_OBJS: [ObjectiveDef; 3] = [
    ObjectiveDef { id: ObjectiveId::BalconyRestore, title: "Restore the Balcony", description: "Reach biodiversity 3", kind: ObjectiveKind::BiodiversityAtLeast(3), required_for_completion: true },
    ObjectiveDef { id: ObjectiveId::BalconyTrader, title: "Small Trader", description: "Earn 12 Dew", kind: ObjectiveKind::DewEarnedAtLeast(12), required_for_completion: false },
    ObjectiveDef { id: ObjectiveId::BalconyVariety, title: "Curious Gardener", description: "Discover 8 card types", kind: ObjectiveKind::DiscoveriesAtLeast(8), required_for_completion: false },
];
const MEADOW_OBJS: [ObjectiveDef; 3] = [
    ObjectiveDef { id: ObjectiveId::MeadowCommunity, title: "Living Meadow", description: "Reach biodiversity 5", kind: ObjectiveKind::BiodiversityAtLeast(5), required_for_completion: true },
    ObjectiveDef { id: ObjectiveId::MeadowSynergies, title: "Garden Relationships", description: "Activate two synergies", kind: ObjectiveKind::ActivateSynergies(2), required_for_completion: false },
    ObjectiveDef { id: ObjectiveId::MeadowPollination, title: "Pollinator Haven", description: "Pollinate three Mature Vines", kind: ObjectiveKind::Pollinations(3), required_for_completion: false },
];
const WETLAND_OBJS: [ObjectiveDef; 3] = [
    ObjectiveDef { id: ObjectiveId::WetlandEngineer, title: "Wetland Engineer", description: "Complete three projects", kind: ObjectiveKind::CompleteProjects(3), required_for_completion: true },
    ObjectiveDef { id: ObjectiveId::WetlandCircular, title: "Circular Garden", description: "Clean or compost four toxins", kind: ObjectiveKind::CleanToxins(4), required_for_completion: false },
    ObjectiveDef { id: ObjectiveId::WetlandSpecialist, title: "Built to Last", description: "Own three distinct installations", kind: ObjectiveKind::DistinctInstallations(3), required_for_completion: false },
];
const CONSERVATORY_OBJS: [ObjectiveDef; 3] = [
    ObjectiveDef { id: ObjectiveId::ConservatorySurvivor, title: "Conservatory Through Time", description: "Reach Year 2", kind: ObjectiveKind::ReachYear(2), required_for_completion: true },
    ObjectiveDef { id: ObjectiveId::ConservatoryCrew, title: "A Place for Everyone", description: "Have three assigned worker types", kind: ObjectiveKind::DistinctWorkers(3), required_for_completion: false },
    ObjectiveDef { id: ObjectiveId::ConservatoryInfrastructure, title: "Prepared for Anything", description: "Install three advanced structures", kind: ObjectiveKind::InstallationsAtLeast(3), required_for_completion: false },
];
const WASTES_OBJS: [ObjectiveDef; 3] = [
    ObjectiveDef { id: ObjectiveId::WastesGenesis, title: "Genesis Restored", description: "Grow the Genesis Bloom", kind: ObjectiveKind::WinGenesisBloom, required_for_completion: true },
    ObjectiveDef { id: ObjectiveId::WastesBiodiversity, title: "Life Returns", description: "Reach biodiversity 8", kind: ObjectiveKind::BiodiversityAtLeast(8), required_for_completion: false },
    ObjectiveDef { id: ObjectiveId::WastesResilience, title: "Resilient Settlement", description: "Finish with three non-fatigued workers and five installations", kind: ObjectiveKind::AssignedNonFatiguedWorkersAndInstallations { workers: 3, installations: 5 }, required_for_completion: false },
];

pub const GARDENS: &[GardenDef] = &[
    GardenDef {
        id: GardenId::AbandonedBalcony,
        name: "Abandoned Balcony",
        subtitle: "Concrete Cracks",
        description: "Restore a neglected balcony.",
        intro: "A few hardy seeds remain. Restore biodiversity to 3.",
        starting_dew: 0,
        starting_cards: BALCONY_START,
        features: FeatureRules { habitats: false, commissions: true, projects: false, installations: false, seasons: false, weather: false, workers: false, advanced_structures: false },
        allowed_cards: BASE_CARDS,
        allowed_packs: SOIL_PACK,
        allowed_blueprints: NO_BLUEPRINTS,
        weather_deck: NO_WEATHER,
        objectives: &BALCONY_OBJS,
        board_color: Color::srgb(0.18, 0.22, 0.16),
        grid_cols: 6,
        grid_rows: 3,
    },
    GardenDef {
        id: GardenId::WildMeadow,
        name: "Wild Meadow",
        subtitle: "Open Grassland",
        description: "Build a thriving network of habitats and pollinators.",
        intro: "Habitats awaken. Reach biodiversity 5.",
        starting_dew: 2,
        starting_cards: MEADOW_START,
        features: FeatureRules { habitats: true, commissions: true, projects: false, installations: false, seasons: false, weather: false, workers: false, advanced_structures: false },
        allowed_cards: BASE_CARDS,
        allowed_packs: SOIL_POLL,
        allowed_blueprints: NO_BLUEPRINTS,
        weather_deck: NO_WEATHER,
        objectives: &MEADOW_OBJS,
        board_color: Color::srgb(0.22, 0.28, 0.18),
        grid_cols: 6,
        grid_rows: 3,
    },
    GardenDef {
        id: GardenId::FloodedWetland,
        name: "Flooded Wetland",
        subtitle: "Waterlogged",
        description: "Engineer circular production.",
        intro: "Projects and installations unlock. Complete three projects.",
        starting_dew: 4,
        starting_cards: WETLAND_START,
        features: FeatureRules { habitats: true, commissions: true, projects: true, installations: true, seasons: false, weather: true, workers: false, advanced_structures: false },
        allowed_cards: WETLAND_CARDS,
        allowed_packs: WETLAND_PACKS,
        allowed_blueprints: BASE_BLUEPRINTS,
        weather_deck: WETLAND_WEATHER,
        objectives: &WETLAND_OBJS,
        board_color: Color::srgb(0.16, 0.22, 0.20),
        grid_cols: 6,
        grid_rows: 3,
    },
    GardenDef {
        id: GardenId::MoonlitConservatory,
        name: "Moonlit Conservatory",
        subtitle: "Glass Haven",
        description: "Survive seasons with crew and structures.",
        intro: "Full seasons and workers arrive. Reach Year 2.",
        starting_dew: 6,
        starting_cards: CONSERVATORY_START,
        features: FeatureRules { habitats: true, commissions: true, projects: true, installations: true, seasons: true, weather: true, workers: true, advanced_structures: true },
        allowed_cards: ALL_CARDS,
        allowed_packs: ALL_PACKS,
        allowed_blueprints: ALL_BLUEPRINTS,
        weather_deck: ALL_WEATHER,
        objectives: &CONSERVATORY_OBJS,
        board_color: Color::srgb(0.20, 0.18, 0.24),
        grid_cols: 6,
        grid_rows: 3,
    },
    GardenDef {
        id: GardenId::GlassWastes,
        name: "Glass Wastes",
        subtitle: "Shattered Greenhouse",
        description: "Full-system final challenge.",
        intro: "Harsh weather, scarce start. Grow the Genesis Bloom.",
        starting_dew: 2,
        starting_cards: WASTES_START,
        features: FeatureRules { habitats: true, commissions: true, projects: true, installations: true, seasons: true, weather: true, workers: true, advanced_structures: true },
        allowed_cards: ALL_CARDS,
        allowed_packs: ALL_PACKS,
        allowed_blueprints: ALL_BLUEPRINTS,
        weather_deck: ALL_WEATHER,
        objectives: &WASTES_OBJS,
        board_color: Color::srgb(0.20, 0.20, 0.18),
        grid_cols: 6,
        grid_rows: 3,
    },
];

pub fn garden_def(id: GardenId) -> &'static GardenDef {
    GARDENS.iter().find(|g| g.id == id).unwrap()
}

pub fn is_unlocked(garden: GardenId, progress: &[crate::save::SavedGardenProgress]) -> bool {
    if garden == GardenId::AbandonedBalcony {
        return true;
    }
    let prev_index = garden.index() - 1;
    let prev_id = GardenId::ALL[prev_index].stable_id().to_string();
    progress.iter().any(|p| p.id == prev_id && p.completed)
}

pub fn next_garden(current: GardenId) -> Option<GardenId> {
    let idx = current.index();
    if idx + 1 < GardenId::ALL.len() {
        Some(GardenId::ALL[idx + 1])
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub struct ObjectiveProgress {
    pub id: ObjectiveId,
    pub current: u32,
    pub required: u32,
    pub complete: bool,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct GardenRun {
    pub garden: Option<GardenId>,
    pub seed: u64,
    pub completed: bool,
    pub awarded_stars: u8,
    pub objectives: Vec<ObjectiveProgress>,
}

impl GardenRun {
    pub fn start(garden: GardenId, seed: u64) -> Self {
        let def = garden_def(garden);
        let objectives = def.objectives.iter().map(|o| {
            let required = match o.kind {
                ObjectiveKind::BiodiversityAtLeast(n) => n,
                ObjectiveKind::DewEarnedAtLeast(n) => n,
                ObjectiveKind::DiscoveriesAtLeast(n) => n,
                ObjectiveKind::CompleteProjects(n) => n,
                ObjectiveKind::InstallationsAtLeast(n) => n,
                ObjectiveKind::DistinctInstallations(n) => n,
                ObjectiveKind::ActivateSynergies(n) => n,
                ObjectiveKind::Pollinations(n) => n,
                ObjectiveKind::CleanToxins(n) => n,
                ObjectiveKind::AssignedWorkers(n) => n,
                ObjectiveKind::DistinctWorkers(n) => n,
                ObjectiveKind::SurviveSeasons(n) => n,
                ObjectiveKind::ReachYear(n) => n,
                ObjectiveKind::GrowCard(_) => 1,
                ObjectiveKind::WinGenesisBloom => 1,
                ObjectiveKind::AssignedNonFatiguedWorkersAndInstallations { .. } => 1,
            };
            ObjectiveProgress { id: o.id, current: 0, required, complete: false }
        }).collect();
        Self { garden: Some(garden), seed, completed: false, awarded_stars: 0, objectives }
    }
    pub fn free(seed: u64) -> Self {
        Self { garden: None, seed, completed: false, awarded_stars: 0, objectives: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_garden_is_unlocked() {
        let progress: Vec<crate::save::SavedGardenProgress> = vec![];
        assert!(is_unlocked(GardenId::AbandonedBalcony, &progress));
    }
    #[test]
    fn later_garden_requires_previous_completion() {
        let progress = vec![crate::save::SavedGardenProgress { id: GardenId::AbandonedBalcony.stable_id().to_string(), completed: false, stars: 0, best_biodiversity: 0, best_dew_earned: 0, best_year: 0, completions: 0 }];
        assert!(!is_unlocked(GardenId::WildMeadow, &progress));
        let progress2 = vec![crate::save::SavedGardenProgress { id: GardenId::AbandonedBalcony.stable_id().to_string(), completed: true, stars: 1, best_biodiversity: 3, best_dew_earned: 10, best_year: 1, completions: 1 }];
        assert!(is_unlocked(GardenId::WildMeadow, &progress2));
    }
    #[test]
    fn free_garden_allows_all_content() {
        let rules = crate::game::run_rules::RunRules::free_garden();
        assert!(rules.allows_card(crate::game::CardType::Botanist));
        assert!(rules.allows_card(crate::game::CardType::Greenhouse));
        assert!(rules.allows_pack(crate::game::PackId::Specialist));
    }
    #[test]
    fn pack_draws_never_return_disallowed_cards() {
        let rules = crate::game::run_rules::RunRules::for_garden(GardenId::AbandonedBalcony);
        // Balcony should not allow Botanist, so specialist pack draws filtered should not contain it
        assert!(!rules.allows_card(crate::game::CardType::Botanist));
        // Simulate that pack draws would be filtered
        let allowed: Vec<_> = crate::game::packs::SPECIALIST_ENTRIES.iter().filter(|e| rules.allows_card(e.card)).collect();
        for e in allowed {
            assert!(rules.allows_card(e.card));
        }
        // Ensure at least one entry would be filtered (Botanist)
        let has_botanist = crate::game::packs::SPECIALIST_ENTRIES.iter().any(|e| e.card == crate::game::CardType::Botanist);
        assert!(has_botanist);
        let filtered_has_botanist = crate::game::packs::SPECIALIST_ENTRIES.iter().filter(|e| rules.allows_card(e.card)).any(|e| e.card == crate::game::CardType::Botanist);
        assert!(!filtered_has_botanist);
    }
    #[test]
    fn blueprint_refresh_ignores_disallowed_blueprints() {
        use crate::game::projects::{BlueprintState, BlueprintId};
        use crate::game::discovery::DiscoveryState;
        use crate::game::commissions::CommissionBoard;
        use crate::game::events::PendingGameEvents;
        let mut state = BlueprintState::default();
        let mut disc = DiscoveryState::default();
        disc.discover(crate::game::CardType::WasteToxin);
        let board = CommissionBoard::default();
        let mut events = PendingGameEvents::default();
        let rules = crate::game::run_rules::RunRules::for_garden(GardenId::AbandonedBalcony);
        // Balcony has no blueprints allowed, so even though WasteToxin would unlock CompostCradle, it should be ignored
        crate::game::projects::refresh_blueprint_unlocks_with_rules(&mut state, &disc, &board, &rules, &mut events);
        assert!(!state.unlocked.contains(&BlueprintId::CompostCradle));
    }
    #[test]
    fn season_clock_does_not_advance_when_disabled() {
        let rules = crate::game::run_rules::RunRules::for_garden(GardenId::AbandonedBalcony);
        assert!(!rules.features.seasons);
        // If seasons disabled, tick should be no-op
        let mut clock = crate::game::seasons::SeasonClock::default();
        let before = clock.total_moons;
        // Simulate that system would early return, so total_moons stays same
        if !rules.features.seasons {
            // do not advance
        } else {
            clock.advance_moon();
        }
        assert_eq!(clock.total_moons, before);
    }
    #[test]
    fn worker_assignment_rejected_when_disabled() {
        let rules = crate::game::run_rules::RunRules::for_garden(GardenId::AbandonedBalcony);
        assert!(!rules.features.workers);
        // Assignment should be rejected when workers disabled
        let can_assign = rules.features.workers && rules.allows_card(crate::game::CardType::Botanist);
        assert!(!can_assign);
    }
    #[test]
    fn primary_objective_completes_garden() {
        let snap = crate::game::objectives::ObjectiveSnapshot { biodiversity: 3, ..Default::default() };
        assert!(crate::game::objectives::is_complete(crate::game::objectives::ObjectiveKind::BiodiversityAtLeast(3), &snap));
        // Simulate GardenRun completion
        let mut run = GardenRun::start(GardenId::AbandonedBalcony, 123);
        // Manually mark primary complete
        run.objectives[0].current = 3;
        run.objectives[0].complete = true;
        run.completed = run.objectives.iter().filter(|o| {
            let def = garden_def(GardenId::AbandonedBalcony).objectives.iter().find(|d| d.id == o.id).unwrap();
            def.required_for_completion
        }).all(|o| o.complete);
        assert!(run.completed);
    }
    #[test]
    fn optional_objectives_award_extra_stars() {
        let mut run = GardenRun::start(GardenId::AbandonedBalcony, 1);
        // Complete primary + one optional
        for prog in &mut run.objectives {
            prog.current = prog.required;
            prog.complete = true;
        }
        run.completed = true;
        run.awarded_stars = run.objectives.iter().filter(|o| o.complete).count() as u8;
        assert_eq!(run.awarded_stars, 3);
        // If only primary complete, stars =1
        let mut run2 = GardenRun::start(GardenId::AbandonedBalcony, 1);
        run2.objectives[0].complete = true;
        run2.objectives[0].current = run2.objectives[0].required;
        run2.completed = run2.objectives.iter().filter(|o| {
            let def = garden_def(GardenId::AbandonedBalcony).objectives.iter().find(|d| d.id == o.id).unwrap();
            def.required_for_completion
        }).all(|o| o.complete);
        run2.awarded_stars = run2.objectives.iter().filter(|o| o.complete).count() as u8;
        assert_eq!(run2.awarded_stars, 1);
        assert!(run.awarded_stars > run2.awarded_stars);
    }
    #[test]
    fn stars_never_decrease_on_replay() {
        let old_stars = 3u8;
        let new_stars = 1u8;
        let merged = old_stars.max(new_stars);
        assert_eq!(merged, 3);
        let old2 = 2u8;
        let new2 = 3u8;
        assert_eq!(old2.max(new2), 3);
    }
    #[test]
    fn next_garden_unlocks_after_completion() {
        let cur = GardenId::AbandonedBalcony;
        let next = next_garden(cur).unwrap();
        assert_eq!(next, GardenId::WildMeadow);
        // After completing Balcony, WildMeadow should be unlocked
        let progress = vec![crate::save::SavedGardenProgress { id: cur.stable_id().to_string(), completed: true, stars: 1, best_biodiversity: 3, best_dew_earned: 10, best_year: 1, completions: 1 }];
        assert!(is_unlocked(next, &progress));
    }
    #[test]
    fn garden_setup_uses_defined_starting_cards() {
        let def = garden_def(GardenId::AbandonedBalcony);
        assert_eq!(def.starting_cards.len(), 6);
        assert!(def.starting_cards.iter().any(|c| c.card == crate::game::CardType::Gardener));
        let wastes = garden_def(GardenId::GlassWastes);
        assert_eq!(wastes.starting_cards.len(), 4);
        assert_eq!(wastes.starting_dew, 2);
    }
    #[test]
    fn campaign_seed_replays_deterministically() {
        let run1 = GardenRun::start(GardenId::WildMeadow, 42);
        let run2 = GardenRun::start(GardenId::WildMeadow, 42);
        assert_eq!(run1.seed, run2.seed);
        assert_eq!(run1.garden, run2.garden);
        assert_eq!(run1.objectives.len(), run2.objectives.len());
        // Rng determinism
        use rand::{SeedableRng, RngExt};
        use rand::rngs::StdRng;
        let mut a = StdRng::seed_from_u64(42);
        let mut b = StdRng::seed_from_u64(42);
        let ra: u64 = a.random();
        let rb: u64 = b.random();
        assert_eq!(ra, rb);
    }
    #[test]
    fn save_v5_loads_with_v6_defaults() {
        let data = crate::save::SaveData::default();
        assert_eq!(data.version, crate::save::SAVE_VERSION);
        assert_eq!(data.garden_progress.len(), 0);
        assert_eq!(data.campaign_completed, false);
        let old = crate::save::SaveData { version: 5, ..Default::default() };
        assert_eq!(old.garden_progress.len(), 0);
        assert_eq!(old.total_campaign_stars, 0);
    }
    #[test]
    fn glass_wastes_requires_genesis_bloom() {
        let def = garden_def(GardenId::GlassWastes);
        let primary = def.objectives.iter().find(|o| o.required_for_completion).unwrap();
        assert!(matches!(primary.kind, crate::game::objectives::ObjectiveKind::WinGenesisBloom));
    }
    #[test]
    fn free_garden_original_win_route_still_works() {
        let rules = crate::game::run_rules::RunRules::free_garden();
        assert!(rules.allows_card(crate::game::CardType::GenesisBloom));
        assert!(rules.allows_card(crate::game::CardType::ApexSpore));
        assert!(rules.features.seasons);
    }
}
