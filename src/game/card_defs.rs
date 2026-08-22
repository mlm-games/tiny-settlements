use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CardType {
    #[default]
    None,
    Gardener,
    BioSubstrate,
    SporePod,
    NutrientSlime,
    BasicFungi,
    ProcessedNutrients,
    VineSeed,
    YoungVine,
    MatureVine,
    FlutterwingSpore,
    FlutterwingLarva,
    MatureFlutterwing,
    FertilizedVinePod,
    SymbioticAlgae,
    LuminaCrystal,
    GrazingSlugEgg,
    GrazingSlug,
    RichMulch,
    FertileSubstrate,
    WasteToxin,
    ApexSpore,
    GrowingApex,
    GenesisBloom,
    NurseryTray,
    CompostCradle,
    MyceliumBed,
    PollinatorLodge,
    DewBasin,
    SeedArchive,
    // Phase 5 workers
    Botanist,
    Mycologist,
    Entomologist,
    CompostKeeper,
    WaterTender,
    // Phase 5 advanced structures
    Greenhouse,
    RainBarrel,
    BeeHotel,
    MushroomCellar,
    ObservationStation,
    IrrigationChannel,
}

impl CardType {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "?",
            Self::Gardener => "Gardener",
            Self::BioSubstrate => "Bio-Substrate",
            Self::SporePod => "Spore Pod",
            Self::NutrientSlime => "Nutrient Slime",
            Self::BasicFungi => "Basic Fungi",
            Self::ProcessedNutrients => "Processed Nutrients",
            Self::VineSeed => "Vine Seed",
            Self::YoungVine => "Young Vine",
            Self::MatureVine => "Mature Vine",
            Self::FlutterwingSpore => "Flutterwing Spore",
            Self::FlutterwingLarva => "Flutterwing Larva",
            Self::MatureFlutterwing => "Mature Flutterwing",
            Self::FertilizedVinePod => "Fertilized Pod",
            Self::SymbioticAlgae => "Symbiotic Algae",
            Self::LuminaCrystal => "Lumina Crystal",
            Self::GrazingSlugEgg => "Slug Egg",
            Self::GrazingSlug => "Grazing Slug",
            Self::RichMulch => "Rich Mulch",
            Self::FertileSubstrate => "Fertile Substrate",
            Self::WasteToxin => "Waste Toxin",
            Self::ApexSpore => "Apex Spore",
            Self::GrowingApex => "Growing Apex",
            Self::GenesisBloom => "Genesis Bloom",
            Self::NurseryTray => "Nursery Tray",
            Self::CompostCradle => "Compost Cradle",
            Self::MyceliumBed => "Mycelium Bed",
            Self::PollinatorLodge => "Pollinator Lodge",
            Self::DewBasin => "Dew Basin",
            Self::SeedArchive => "Seed Archive",
            Self::Botanist => "Botanist",
            Self::Mycologist => "Mycologist",
            Self::Entomologist => "Entomologist",
            Self::CompostKeeper => "Compost Keeper",
            Self::WaterTender => "Water Tender",
            Self::Greenhouse => "Greenhouse",
            Self::RainBarrel => "Rain Barrel",
            Self::BeeHotel => "Bee Hotel",
            Self::MushroomCellar => "Mushroom Cellar",
            Self::ObservationStation => "Observation Station",
            Self::IrrigationChannel => "Irrigation Channel",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Gardener => Color::srgb(0.85, 0.75, 0.45),
            Self::BioSubstrate => Color::srgb(0.45, 0.35, 0.25),
            Self::FertileSubstrate => Color::srgb(0.35, 0.5, 0.22),
            Self::SporePod | Self::VineSeed | Self::FlutterwingSpore | Self::ApexSpore => {
                Color::srgb(0.55, 0.72, 0.4)
            }
            Self::NutrientSlime => Color::srgb(0.35, 0.78, 0.45),
            Self::ProcessedNutrients => Color::srgb(0.3, 0.65, 0.55),
            Self::LuminaCrystal => Color::srgb(0.45, 0.78, 0.95),
            Self::BasicFungi => Color::srgb(0.72, 0.55, 0.4),
            Self::YoungVine | Self::MatureVine => Color::srgb(0.32, 0.72, 0.35),
            Self::FlutterwingLarva => Color::srgb(0.75, 0.55, 0.85),
            Self::MatureFlutterwing => Color::srgb(0.7, 0.48, 0.95),
            Self::FertilizedVinePod => Color::srgb(0.55, 0.65, 0.3),
            Self::SymbioticAlgae => Color::srgb(0.25, 0.78, 0.6),
            Self::GrazingSlugEgg => Color::srgb(0.65, 0.55, 0.45),
            Self::GrazingSlug => Color::srgb(0.55, 0.45, 0.55),
            Self::RichMulch => Color::srgb(0.4, 0.3, 0.2),
            Self::WasteToxin => Color::srgb(0.75, 0.22, 0.25),
            Self::GrowingApex => Color::srgb(0.55, 0.88, 0.7),
            Self::GenesisBloom => Color::srgb(0.55, 0.95, 0.75),
            Self::NurseryTray => Color::srgb(0.55, 0.65, 0.45),
            Self::CompostCradle => Color::srgb(0.45, 0.45, 0.35),
            Self::MyceliumBed => Color::srgb(0.68, 0.62, 0.48),
            Self::PollinatorLodge => Color::srgb(0.72, 0.68, 0.52),
            Self::DewBasin => Color::srgb(0.45, 0.68, 0.78),
            Self::SeedArchive => Color::srgb(0.60, 0.58, 0.42),
            Self::Botanist => Color::srgb(0.52, 0.72, 0.55),
            Self::Mycologist => Color::srgb(0.65, 0.60, 0.45),
            Self::Entomologist => Color::srgb(0.70, 0.65, 0.80),
            Self::CompostKeeper => Color::srgb(0.55, 0.50, 0.35),
            Self::WaterTender => Color::srgb(0.50, 0.65, 0.85),
            Self::Greenhouse => Color::srgb(0.75, 0.85, 0.70),
            Self::RainBarrel => Color::srgb(0.45, 0.70, 0.85),
            Self::BeeHotel => Color::srgb(0.78, 0.72, 0.45),
            Self::MushroomCellar => Color::srgb(0.55, 0.50, 0.65),
            Self::ObservationStation => Color::srgb(0.60, 0.65, 0.75),
            Self::IrrigationChannel => Color::srgb(0.50, 0.75, 0.65),
            Self::None => Color::srgb(0.3, 0.3, 0.3),
        }
    }

    pub fn is_plant(self) -> bool {
        matches!(
            self,
            Self::BasicFungi
                | Self::YoungVine
                | Self::MatureVine
                | Self::SymbioticAlgae
                | Self::GrowingApex
        )
    }

    pub fn is_seed_or_spore(self) -> bool {
        matches!(
            self,
            Self::SporePod
                | Self::VineSeed
                | Self::FlutterwingSpore
                | Self::GrazingSlugEgg
                | Self::FertilizedVinePod
                | Self::ApexSpore
        )
    }

    pub fn is_nutrient(self) -> bool {
        matches!(
            self,
            Self::NutrientSlime | Self::ProcessedNutrients | Self::LuminaCrystal
        )
    }

    pub fn is_substrate(self) -> bool {
        matches!(self, Self::BioSubstrate | Self::FertileSubstrate)
    }

    pub fn is_mature_species(self) -> bool {
        matches!(
            self,
            Self::BasicFungi
                | Self::MatureVine
                | Self::MatureFlutterwing
                | Self::SymbioticAlgae
                | Self::GrazingSlug
                | Self::GenesisBloom
        )
    }

    pub fn needs_nutrient(self) -> Option<Self> {
        match self {
            Self::VineSeed | Self::YoungVine => Some(Self::ProcessedNutrients),
            Self::FlutterwingSpore => Some(Self::NutrientSlime),
            Self::ApexSpore => Some(Self::LuminaCrystal),
            _ => None,
        }
    }

    pub fn needs_substrate(self) -> Option<Self> {
        match self {
            Self::SymbioticAlgae | Self::ApexSpore => Some(Self::FertileSubstrate),
            _ => None,
        }
    }

    pub fn produces_passively(self) -> Option<(Self, f32)> {
        match self {
            Self::BasicFungi => Some((Self::ProcessedNutrients, 15.0)),
            Self::SymbioticAlgae => Some((Self::LuminaCrystal, 20.0)),
            Self::GrazingSlug => Some((Self::RichMulch, 10.0)),
            _ => None,
        }
    }

    pub fn eats(self) -> Option<Self> {
        (self == Self::GrazingSlug).then_some(Self::BasicFungi)
    }

    pub fn needs_nearby(self) -> Option<Self> {
        (self == Self::GrazingSlugEgg).then_some(Self::BasicFungi)
    }

    pub fn next_growth(self) -> Option<Self> {
        match self {
            Self::SporePod => Some(Self::BasicFungi),
            Self::VineSeed => Some(Self::YoungVine),
            Self::YoungVine => Some(Self::MatureVine),
            Self::FlutterwingSpore => Some(Self::FlutterwingLarva),
            Self::FlutterwingLarva => Some(Self::MatureFlutterwing),
            Self::FertilizedVinePod => Some(Self::SymbioticAlgae),
            Self::ApexSpore => Some(Self::GrowingApex),
            Self::GrowingApex => Some(Self::GenesisBloom),
            _ => None,
        }
    }

    /// Seconds for a growth tick once conditions are met.
    pub fn growth_duration(self) -> Option<f32> {
        match self {
            Self::SporePod => Some(5.0),
            Self::FlutterwingLarva => Some(10.0),
            Self::VineSeed
            | Self::YoungVine
            | Self::FlutterwingSpore
            | Self::FertilizedVinePod
            | Self::ApexSpore
            | Self::GrowingApex => Some(8.0),
            _ => None,
        }
    }

    /// True if this stage auto-grows without a nutrient application.
    pub fn auto_grows(self) -> bool {
        matches!(
            self,
            Self::SporePod | Self::FlutterwingLarva | Self::FertilizedVinePod | Self::GrowingApex
        )
    }

    pub const fn sell_value(self) -> Option<u32> {
        match self {
            Self::BioSubstrate => Some(1),
            Self::SporePod => Some(2),
            Self::NutrientSlime => Some(1),
            Self::ProcessedNutrients => Some(2),
            Self::VineSeed => Some(3),
            Self::FlutterwingSpore => Some(4),
            Self::FertilizedVinePod => Some(5),
            Self::LuminaCrystal => Some(5),
            Self::GrazingSlugEgg => Some(4),
            Self::RichMulch => Some(3),
            Self::WasteToxin => Some(0),
            Self::NurseryTray => Some(3),
            Self::CompostCradle => Some(4),
            Self::MyceliumBed => Some(5),
            Self::PollinatorLodge => Some(6),
            Self::DewBasin => Some(7),
            Self::SeedArchive => Some(8),
            Self::Botanist => Some(5),
            Self::Mycologist => Some(5),
            Self::Entomologist => Some(6),
            Self::CompostKeeper => Some(5),
            Self::WaterTender => Some(5),
            Self::Greenhouse => Some(7),
            Self::RainBarrel => Some(6),
            Self::BeeHotel => Some(6),
            Self::MushroomCellar => Some(6),
            Self::ObservationStation => Some(8),
            Self::IrrigationChannel => Some(7),
            Self::Gardener
            | Self::BasicFungi
            | Self::YoungVine
            | Self::MatureVine
            | Self::FlutterwingLarva
            | Self::MatureFlutterwing
            | Self::SymbioticAlgae
            | Self::GrazingSlug
            | Self::FertileSubstrate
            | Self::ApexSpore
            | Self::GrowingApex
            | Self::GenesisBloom
            | Self::None => None,
        }
    }

    pub const fn is_installation(self) -> bool {
        matches!(
            self,
            Self::NurseryTray
                | Self::CompostCradle
                | Self::MyceliumBed
                | Self::PollinatorLodge
                | Self::DewBasin
                | Self::SeedArchive
                | Self::Greenhouse
                | Self::RainBarrel
                | Self::BeeHotel
                | Self::MushroomCellar
                | Self::ObservationStation
                | Self::IrrigationChannel
        )
    }

    pub const fn is_worker(self) -> bool {
        matches!(
            self,
            Self::Botanist
                | Self::Mycologist
                | Self::Entomologist
                | Self::CompostKeeper
                | Self::WaterTender
        )
    }

    pub const fn is_advanced_structure(self) -> bool {
        matches!(
            self,
            Self::Greenhouse
                | Self::RainBarrel
                | Self::BeeHotel
                | Self::MushroomCellar
                | Self::ObservationStation
                | Self::IrrigationChannel
        )
    }

    pub const fn installation_sell_value(self) -> Option<u32> {
        match self {
            Self::NurseryTray => Some(3),
            Self::CompostCradle => Some(4),
            Self::MyceliumBed => Some(5),
            Self::PollinatorLodge => Some(6),
            Self::DewBasin => Some(7),
            Self::SeedArchive => Some(8),
            Self::Greenhouse => Some(7),
            Self::RainBarrel => Some(6),
            Self::BeeHotel => Some(6),
            Self::MushroomCellar => Some(6),
            Self::ObservationStation => Some(8),
            Self::IrrigationChannel => Some(7),
            _ => None,
        }
    }

    pub fn stable_id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Gardener => "gardener",
            Self::BioSubstrate => "bio_substrate",
            Self::SporePod => "spore_pod",
            Self::NutrientSlime => "nutrient_slime",
            Self::BasicFungi => "basic_fungi",
            Self::ProcessedNutrients => "processed_nutrients",
            Self::VineSeed => "vine_seed",
            Self::YoungVine => "young_vine",
            Self::MatureVine => "mature_vine",
            Self::FlutterwingSpore => "flutterwing_spore",
            Self::FlutterwingLarva => "flutterwing_larva",
            Self::MatureFlutterwing => "mature_flutterwing",
            Self::FertilizedVinePod => "fertilized_vine_pod",
            Self::SymbioticAlgae => "symbiotic_algae",
            Self::LuminaCrystal => "lumina_crystal",
            Self::GrazingSlugEgg => "grazing_slug_egg",
            Self::GrazingSlug => "grazing_slug",
            Self::RichMulch => "rich_mulch",
            Self::FertileSubstrate => "fertile_substrate",
            Self::WasteToxin => "waste_toxin",
            Self::ApexSpore => "apex_spore",
            Self::GrowingApex => "growing_apex",
            Self::GenesisBloom => "genesis_bloom",
            Self::NurseryTray => "nursery_tray",
            Self::CompostCradle => "compost_cradle",
            Self::MyceliumBed => "mycelium_bed",
            Self::PollinatorLodge => "pollinator_lodge",
            Self::DewBasin => "dew_basin",
            Self::SeedArchive => "seed_archive",
            Self::Botanist => "botanist",
            Self::Mycologist => "mycologist",
            Self::Entomologist => "entomologist",
            Self::CompostKeeper => "compost_keeper",
            Self::WaterTender => "water_tender",
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
            "none" => Self::None,
            "gardener" => Self::Gardener,
            "bio_substrate" => Self::BioSubstrate,
            "spore_pod" => Self::SporePod,
            "nutrient_slime" => Self::NutrientSlime,
            "basic_fungi" => Self::BasicFungi,
            "processed_nutrients" => Self::ProcessedNutrients,
            "vine_seed" => Self::VineSeed,
            "young_vine" => Self::YoungVine,
            "mature_vine" => Self::MatureVine,
            "flutterwing_spore" => Self::FlutterwingSpore,
            "flutterwing_larva" => Self::FlutterwingLarva,
            "mature_flutterwing" => Self::MatureFlutterwing,
            "fertilized_vine_pod" => Self::FertilizedVinePod,
            "symbiotic_algae" => Self::SymbioticAlgae,
            "lumina_crystal" => Self::LuminaCrystal,
            "grazing_slug_egg" => Self::GrazingSlugEgg,
            "grazing_slug" => Self::GrazingSlug,
            "rich_mulch" => Self::RichMulch,
            "fertile_substrate" => Self::FertileSubstrate,
            "waste_toxin" => Self::WasteToxin,
            "apex_spore" => Self::ApexSpore,
            "growing_apex" => Self::GrowingApex,
            "genesis_bloom" => Self::GenesisBloom,
            "nursery_tray" => Self::NurseryTray,
            "compost_cradle" => Self::CompostCradle,
            "mycelium_bed" => Self::MyceliumBed,
            "pollinator_lodge" => Self::PollinatorLodge,
            "dew_basin" => Self::DewBasin,
            "seed_archive" => Self::SeedArchive,
            "botanist" => Self::Botanist,
            "mycologist" => Self::Mycologist,
            "entomologist" => Self::Entomologist,
            "compost_keeper" => Self::CompostKeeper,
            "water_tender" => Self::WaterTender,
            "greenhouse" => Self::Greenhouse,
            "rain_barrel" => Self::RainBarrel,
            "bee_hotel" => Self::BeeHotel,
            "mushroom_cellar" => Self::MushroomCellar,
            "observation_station" => Self::ObservationStation,
            "irrigation_channel" => Self::IrrigationChannel,
            _ => return None,
        })
    }

    /// Texture path under `assets/`, when card art exists for this type.
    #[allow(dead_code)]
    pub fn asset_path(self) -> Option<&'static str> {
        Some(match self {
            Self::None | Self::WasteToxin => return None,
            Self::Gardener => "images/cards/gardener.ren",
            Self::BioSubstrate => "images/cards/bio_substrate.ren",
            Self::SporePod => "images/cards/spore_pod.ren",
            Self::NutrientSlime => "images/cards/nutrient_slime.ren",
            Self::BasicFungi => "images/cards/basic_fungi.ren",
            Self::ProcessedNutrients => "images/cards/processed_nutrients.ren",
            Self::VineSeed => "images/cards/vine_seed.ren",
            Self::YoungVine => "images/cards/young_vine.ren",
            Self::MatureVine => "images/cards/mature_vine.ren",
            Self::FlutterwingSpore => "images/cards/flutterwing_spore.ren",
            Self::FlutterwingLarva => "images/cards/flutterwing_larva.ren",
            Self::MatureFlutterwing => "images/cards/mature_flutterwing.ren",
            Self::FertilizedVinePod => "images/cards/fertilized_vine_pod.ren",
            Self::SymbioticAlgae => "images/cards/symbiotic_algae.ren",
            Self::LuminaCrystal => "images/cards/lumina_crystal.ren",
            Self::GrazingSlugEgg => "images/cards/grazing_slug_egg.ren",
            Self::GrazingSlug => "images/cards/grazing_slug.ren",
            Self::RichMulch => "images/cards/rich_mulch.ren",
            Self::FertileSubstrate => "images/cards/fertile_substrate.ren",
            Self::ApexSpore => "images/cards/apex_spore.ren",
            Self::GrowingApex => "images/cards/growing_apex.ren",
            Self::GenesisBloom => "images/cards/genesis_bloom.ren",
            Self::NurseryTray => "images/cards/nursery_tray.ren",
            Self::CompostCradle => "images/cards/compost_cradle.ren",
            Self::MyceliumBed => "images/cards/mycelium_bed.ren",
            Self::PollinatorLodge => "images/cards/pollinator_lodge.ren",
            Self::DewBasin => "images/cards/dew_basin.ren",
            Self::SeedArchive => "images/cards/seed_archive.ren",
            Self::Botanist => "images/cards/botanist.ren",
            Self::Mycologist => "images/cards/mycologist.ren",
            Self::Entomologist => "images/cards/entomologist.ren",
            Self::CompostKeeper => "images/cards/compost_keeper.ren",
            Self::WaterTender => "images/cards/water_tender.ren",
            Self::Greenhouse => "images/cards/greenhouse.ren",
            Self::RainBarrel => "images/cards/rain_barrel.ren",
            Self::BeeHotel => "images/cards/bee_hotel.ren",
            Self::MushroomCellar => "images/cards/mushroom_cellar.ren",
            Self::ObservationStation => "images/cards/observation_station.ren",
            Self::IrrigationChannel => "images/cards/irrigation_channel.ren",
        })
    }
}

#[derive(Clone, Debug)]
pub enum GardenerAction {
    Plant,
    ApplyNutrient { source: Entity },
    Clean,
    UpgradeSubstrate { source: Entity },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassiveKind {
    Produce,
    Pollinate,
    Hatch,
    Eat,
    Grow,
}
