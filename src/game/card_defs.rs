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
