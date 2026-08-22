use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

use super::CardType;

/// Kind of commission objective.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommissionKind {
    OwnCount { card: CardType, need: u32 },
    Biodiversity { need: u32 },
    Grow { card: CardType },
    ProduceCount { card: CardType, need: u32 },
    Pollinate,
    Hatch { card: CardType },
    CleanToxins { need: u32 },
    Create { card: CardType },
    // Phase 3
    CompleteProjects { need: u32 },
    InstallStructures { need: u32 },
    OwnDistinctInstallations { need: u32 },
    CompostToxins { need: u32 },
}

#[derive(Clone, Debug)]
pub struct CommissionTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub kind: CommissionKind,
    pub reward_dew: u32,
}

// Static catalog of 10 templates (spec). Additional can be added later.
pub const COMMISSION_TEMPLATES: &[CommissionTemplate] = &[
    CommissionTemplate {
        id: "forest_floor",
        title: "Forest Floor",
        kind: CommissionKind::OwnCount {
            card: CardType::BasicFungi,
            need: 2,
        },
        reward_dew: 4,
    },
    CommissionTemplate {
        id: "healthy_variety",
        title: "Healthy Variety",
        kind: CommissionKind::Biodiversity { need: 3 },
        reward_dew: 5,
    },
    CommissionTemplate {
        id: "vine_nursery",
        title: "Vine Nursery",
        kind: CommissionKind::Grow {
            card: CardType::MatureVine,
        },
        reward_dew: 6,
    },
    CommissionTemplate {
        id: "pollinator_haven",
        title: "Pollinator Haven",
        kind: CommissionKind::Pollinate,
        reward_dew: 7,
    },
    CommissionTemplate {
        id: "nutrient_reserve",
        title: "Nutrient Reserve",
        kind: CommissionKind::ProduceCount {
            card: CardType::ProcessedNutrients,
            need: 4,
        },
        reward_dew: 5,
    },
    CommissionTemplate {
        id: "slug_sanctuary",
        title: "Slug Sanctuary",
        kind: CommissionKind::Hatch {
            card: CardType::GrazingSlug,
        },
        reward_dew: 7,
    },
    CommissionTemplate {
        id: "clean_garden",
        title: "Clean Garden",
        kind: CommissionKind::CleanToxins { need: 2 },
        reward_dew: 8,
    },
    CommissionTemplate {
        id: "living_soil",
        title: "Living Soil",
        kind: CommissionKind::Create {
            card: CardType::FertileSubstrate,
        },
        reward_dew: 7,
    },
    CommissionTemplate {
        id: "symbiosis_study",
        title: "Symbiosis Study",
        kind: CommissionKind::Grow {
            card: CardType::SymbioticAlgae,
        },
        reward_dew: 10,
    },
    CommissionTemplate {
        id: "rare_light",
        title: "Rare Light",
        kind: CommissionKind::ProduceCount {
            card: CardType::LuminaCrystal,
            need: 2,
        },
        reward_dew: 10,
    },
    CommissionTemplate {
        id: "groundwork",
        title: "Groundwork",
        kind: CommissionKind::CompleteProjects { need: 1 },
        reward_dew: 6,
    },
    CommissionTemplate {
        id: "designed_habitat",
        title: "Designed Habitat",
        kind: CommissionKind::InstallStructures { need: 1 },
        reward_dew: 7,
    },
    CommissionTemplate {
        id: "garden_engineer",
        title: "Garden Engineer",
        kind: CommissionKind::OwnDistinctInstallations { need: 3 },
        reward_dew: 10,
    },
    CommissionTemplate {
        id: "circular_garden",
        title: "Circular Garden",
        kind: CommissionKind::CompostToxins { need: 2 },
        reward_dew: 9,
    },
    CommissionTemplate {
        id: "spring_nursery",
        title: "Spring Nursery",
        kind: CommissionKind::Grow {
            card: CardType::YoungVine,
        },
        reward_dew: 7,
    },
    CommissionTemplate {
        id: "summer_reserves",
        title: "Summer Reserves",
        kind: CommissionKind::ProduceCount {
            card: CardType::ProcessedNutrients,
            need: 10,
        },
        reward_dew: 8,
    },
    CommissionTemplate {
        id: "autumn_harvest",
        title: "Autumn Harvest",
        kind: CommissionKind::OwnCount {
            card: CardType::MatureVine,
            need: 4,
        },
        reward_dew: 10,
    },
    CommissionTemplate {
        id: "winter_keeper",
        title: "Winter Keeper",
        kind: CommissionKind::CompleteProjects { need: 1 },
        reward_dew: 9,
    },
    CommissionTemplate {
        id: "blight_resistant",
        title: "Blight Resistant",
        kind: CommissionKind::CleanToxins { need: 1 },
        reward_dew: 12,
    },
];

#[derive(Clone, Debug)]
pub struct ActiveCommission {
    pub template_id: &'static str,
    pub title: &'static str,
    pub kind: CommissionKind,
    pub reward_dew: u32,
    pub progress: u32,
    pub need: u32,
    pub completed: bool,
}

impl ActiveCommission {
    pub fn from_template(t: &CommissionTemplate) -> Self {
        let need = match &t.kind {
            CommissionKind::OwnCount { need, .. } => *need,
            CommissionKind::Biodiversity { need } => *need,
            CommissionKind::Grow { .. } => 1,
            CommissionKind::ProduceCount { need, .. } => *need,
            CommissionKind::Pollinate => 1,
            CommissionKind::Hatch { .. } => 1,
            CommissionKind::CleanToxins { need } => *need,
            CommissionKind::Create { .. } => 1,
            CommissionKind::CompleteProjects { need } => *need,
            CommissionKind::InstallStructures { need } => *need,
            CommissionKind::OwnDistinctInstallations { need } => *need,
            CommissionKind::CompostToxins { need } => *need,
        };
        Self {
            template_id: t.id,
            title: t.title,
            kind: t.kind.clone(),
            reward_dew: t.reward_dew,
            progress: 0,
            need,
            completed: false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.need
    }
}

#[derive(Resource, Default)]
pub struct CommissionBoard {
    pub active: Vec<ActiveCommission>,
    pub total_completed: u32,
}

impl CommissionBoard {
    pub fn init_with_rng(&mut self, rng: &mut StdRng) {
        self.active.clear();
        let mut pool: Vec<&CommissionTemplate> = COMMISSION_TEMPLATES.iter().collect();
        pool.shuffle(rng);
        for t in pool.into_iter().take(3) {
            self.active.push(ActiveCommission::from_template(t));
        }
    }

    pub fn replace_completed(&mut self, rng: &mut StdRng, idx: usize) {
        if idx >= self.active.len() {
            return;
        }
        let used: std::collections::HashSet<&str> =
            self.active.iter().map(|a| a.template_id).collect();
        let mut candidates: Vec<&CommissionTemplate> = COMMISSION_TEMPLATES
            .iter()
            .filter(|t| !used.contains(t.id))
            .collect();
        if candidates.is_empty() {
            // fallback: allow recycling
            candidates = COMMISSION_TEMPLATES.iter().collect();
        }
        candidates.shuffle(rng);
        if let Some(next) = candidates.first() {
            self.active[idx] = ActiveCommission::from_template(next);
        }
    }
}

/// Snapshot of run state relevant to commission progress.
#[derive(Default)]
pub struct CommissionStateSnapshot {
    pub live_counts: std::collections::HashMap<CardType, u32>,
    pub biodiversity: u32,
    pub produced_counts: std::collections::HashMap<CardType, u32>,
    pub pollinations: u32,
    pub hatched: std::collections::HashMap<CardType, u32>,
    pub cleaned_toxins: u32,
    pub created: std::collections::HashMap<CardType, u32>,
    // Phase 3
    pub projects_completed: u32,
    pub installations_installed: u32,
    pub distinct_installations: u32,
    pub composted_toxins: u32,
}

pub fn progress_for_kind(kind: &CommissionKind, snap: &CommissionStateSnapshot) -> u32 {
    match kind {
        CommissionKind::OwnCount { card, need } => {
            (*snap.live_counts.get(card).unwrap_or(&0)).min(*need)
        }
        CommissionKind::Biodiversity { need } => snap.biodiversity.min(*need),
        CommissionKind::Grow { card } | CommissionKind::Create { card } => {
            if snap.created.get(card).copied().unwrap_or(0) > 0
                || snap.live_counts.get(card).copied().unwrap_or(0) > 0
            {
                1
            } else {
                0
            }
        }
        CommissionKind::ProduceCount { card, need } => {
            (*snap.produced_counts.get(card).unwrap_or(&0)).min(*need)
        }
        CommissionKind::Pollinate => snap.pollinations.min(1),
        CommissionKind::Hatch { card } => {
            if snap.hatched.get(card).copied().unwrap_or(0) > 0 {
                1
            } else {
                0
            }
        }
        CommissionKind::CleanToxins { need } => snap.cleaned_toxins.min(*need),
        CommissionKind::CompleteProjects { need } => snap.projects_completed.min(*need),
        CommissionKind::InstallStructures { need } => snap.installations_installed.min(*need),
        CommissionKind::OwnDistinctInstallations { need } => {
            snap.distinct_installations.min(*need)
        }
        CommissionKind::CompostToxins { need } => snap.composted_toxins.min(*need),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_from_template_need() {
        let t = &COMMISSION_TEMPLATES[0];
        let a = ActiveCommission::from_template(t);
        assert_eq!(a.need, 2);
        assert_eq!(a.reward_dew, 4);
    }

    #[test]
    fn board_init_three_unique() {
        let mut rng = StdRng::seed_from_u64(123);
        let mut board = CommissionBoard::default();
        board.init_with_rng(&mut rng);
        assert_eq!(board.active.len(), 3);
        let mut ids: Vec<&str> = board.active.iter().map(|a| a.template_id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn progress_snapshot() {
        let mut snap = CommissionStateSnapshot::default();
        snap.live_counts.insert(CardType::BasicFungi, 2);
        let kind = CommissionKind::OwnCount {
            card: CardType::BasicFungi,
            need: 2,
        };
        assert_eq!(progress_for_kind(&kind, &snap), 2);
        snap.biodiversity = 3;
        assert_eq!(
            progress_for_kind(&CommissionKind::Biodiversity { need: 3 }, &snap),
            3
        );
        snap.pollinations = 1;
        assert_eq!(
            progress_for_kind(&CommissionKind::Pollinate, &snap),
            1
        );
    }
}
