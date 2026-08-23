use bevy::prelude::*;
use super::CardType;
use super::campaign::GardenId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectiveId {
    BalconyRestore,
    BalconyTrader,
    BalconyVariety,
    MeadowCommunity,
    MeadowSynergies,
    MeadowPollination,
    WetlandEngineer,
    WetlandCircular,
    WetlandSpecialist,
    ConservatorySurvivor,
    ConservatoryCrew,
    ConservatoryInfrastructure,
    WastesGenesis,
    WastesBiodiversity,
    WastesResilience,
}

#[derive(Clone, Copy, Debug)]
pub enum ObjectiveKind {
    BiodiversityAtLeast(u32),
    DewEarnedAtLeast(u32),
    DiscoveriesAtLeast(u32),
    CompleteProjects(u32),
    InstallationsAtLeast(u32),
    DistinctInstallations(u32),
    ActivateSynergies(u32),
    Pollinations(u32),
    CleanToxins(u32),
    AssignedWorkers(u32),
    DistinctWorkers(u32),
    SurviveSeasons(u32),
    ReachYear(u32),
    GrowCard(CardType),
    WinGenesisBloom,
    AssignedNonFatiguedWorkersAndInstallations {
        workers: u32,
        installations: u32,
    },
}

pub struct ObjectiveDef {
    pub id: ObjectiveId,
    pub title: &'static str,
    pub description: &'static str,
    pub kind: ObjectiveKind,
    pub required_for_completion: bool,
}

// Snapshot for evaluation
#[derive(Clone, Default)]
pub struct ObjectiveSnapshot {
    pub biodiversity: u32,
    pub dew_earned: u32,
    pub discoveries: u32,
    pub projects_completed: u32,
    pub installations: u32,
    pub distinct_installations: u32,
    pub synergies_activated: u32,
    pub pollinations: u32,
    pub cleaned_toxins: u32,
    pub assigned_workers: u32,
    pub assigned_non_fatigued_workers: u32,
    pub distinct_workers: u32,
    pub seasons_survived: u32,
    pub current_year: u32,
    pub has_genesis: bool,
    pub grown_cards: std::collections::HashSet<CardType>,
}

pub fn objective_title(id: ObjectiveId) -> &'static str {
    match id {
        ObjectiveId::BalconyRestore => "Restore the Balcony",
        ObjectiveId::BalconyTrader => "Small Trader",
        ObjectiveId::BalconyVariety => "Curious Gardener",
        ObjectiveId::MeadowCommunity => "Living Meadow",
        ObjectiveId::MeadowSynergies => "Garden Relationships",
        ObjectiveId::MeadowPollination => "Pollinator Haven",
        ObjectiveId::WetlandEngineer => "Wetland Engineer",
        ObjectiveId::WetlandCircular => "Circular Garden",
        ObjectiveId::WetlandSpecialist => "Built to Last",
        ObjectiveId::ConservatorySurvivor => "Conservatory Through Time",
        ObjectiveId::ConservatoryCrew => "A Place for Everyone",
        ObjectiveId::ConservatoryInfrastructure => "Prepared for Anything",
        ObjectiveId::WastesGenesis => "Genesis Restored",
        ObjectiveId::WastesBiodiversity => "Life Returns",
        ObjectiveId::WastesResilience => "Resilient Settlement",
    }
}

pub fn progress_for_objective(kind: ObjectiveKind, snap: &ObjectiveSnapshot) -> (u32, u32) {
    let (current, required) = match kind {
        ObjectiveKind::BiodiversityAtLeast(n) => (snap.biodiversity.min(n), n),
        ObjectiveKind::DewEarnedAtLeast(n) => (snap.dew_earned.min(n), n),
        ObjectiveKind::DiscoveriesAtLeast(n) => (snap.discoveries.min(n), n),
        ObjectiveKind::CompleteProjects(n) => (snap.projects_completed.min(n), n),
        ObjectiveKind::InstallationsAtLeast(n) => (snap.installations.min(n), n),
        ObjectiveKind::DistinctInstallations(n) => (snap.distinct_installations.min(n), n),
        ObjectiveKind::ActivateSynergies(n) => (snap.synergies_activated.min(n), n),
        ObjectiveKind::Pollinations(n) => (snap.pollinations.min(n), n),
        ObjectiveKind::CleanToxins(n) => (snap.cleaned_toxins.min(n), n),
        ObjectiveKind::AssignedWorkers(n) => (snap.assigned_workers.min(n), n),
        ObjectiveKind::DistinctWorkers(n) => (snap.distinct_workers.min(n), n),
        ObjectiveKind::SurviveSeasons(n) => (snap.seasons_survived.min(n), n),
        ObjectiveKind::ReachYear(n) => ((snap.current_year >= n) as u32, 1),
        ObjectiveKind::GrowCard(card) => ((snap.grown_cards.contains(&card)) as u32, 1),
        ObjectiveKind::WinGenesisBloom => ((snap.has_genesis) as u32, 1),
        ObjectiveKind::AssignedNonFatiguedWorkersAndInstallations { workers, installations } => {
            let cur_workers = snap.assigned_non_fatigued_workers.min(workers);
            let cur_installs = snap.installations.min(installations);
            // require both; progress is min of ratios mapped to single bar (use workers as gate + installations as secondary)
            // Show combined requires: complete only when both satisfied
            let cur = if snap.assigned_non_fatigued_workers >= workers && snap.installations >= installations { 1 } else { 0 };
            let req = 1;
            // For display we want current as number of satisfied sub-goals? But UI takes current/required directly from this fn.
            // To show granular progress, return workers satisfied + installs satisfied weighted.
            // Simpler: return 1/1 when complete else 0/1 (binary) but keep required fields updated via current check
            let _ = (cur_workers, cur_installs);
            (cur, req)
        }
    };
    (current, required)
}

pub fn is_complete(kind: ObjectiveKind, snap: &ObjectiveSnapshot) -> bool {
    let (cur, req) = progress_for_objective(kind, snap);
    cur >= req
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn objective_progress_uses_existing_run_counters() {
        let mut snap = ObjectiveSnapshot::default();
        snap.biodiversity = 3;
        let (cur, req) = progress_for_objective(ObjectiveKind::BiodiversityAtLeast(3), &snap);
        assert_eq!(cur, 3);
        assert_eq!(req, 3);
        assert!(is_complete(ObjectiveKind::BiodiversityAtLeast(3), &snap));
        snap.biodiversity = 2;
        assert!(!is_complete(ObjectiveKind::BiodiversityAtLeast(3), &snap));
    }
    #[test]
    fn primary_objective_completes_garden() {
        let snap = ObjectiveSnapshot { biodiversity: 3, ..Default::default() };
        assert!(is_complete(ObjectiveKind::BiodiversityAtLeast(3), &snap));
    }
}
