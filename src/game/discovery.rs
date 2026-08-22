use std::collections::HashSet;

use bevy::prelude::*;

use super::CardType;

/// Tracks which card types have ever been seen in this save / run.
#[derive(Resource, Default)]
pub struct DiscoveryState {
    pub discovered: HashSet<CardType>,
}

impl DiscoveryState {
    pub fn discover(&mut self, card: CardType) -> bool {
        if card == CardType::None {
            return false;
        }
        self.discovered.insert(card)
    }

    pub fn count(&self) -> u16 {
        self.discovered.len() as u16
    }

    pub fn contains(&self, card: CardType) -> bool {
        self.discovered.contains(&card)
    }

    pub fn to_id_strings(&self) -> Vec<String> {
        let mut v: Vec<String> = self.discovered.iter().map(|c| c.stable_id().to_string()).collect();
        v.sort();
        v
    }

    pub fn from_id_strings(ids: &[String]) -> Self {
        let mut s = HashSet::new();
        for id in ids {
            if let Some(c) = CardType::from_stable_id(id) {
                s.insert(c);
            }
        }
        Self { discovered: s }
    }

    pub fn total_unique_cards() -> u32 {
        // Count of distinct CardType variants excluding None (used for HUD denominator).
        // Keep in sync with CardType enum.
        23
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_insert_and_count() {
        let mut d = DiscoveryState::default();
        assert!(d.discover(CardType::BioSubstrate));
        assert!(!d.discover(CardType::BioSubstrate)); // second time not new
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn discovery_roundtrip_ids() {
        let mut d = DiscoveryState::default();
        d.discover(CardType::BioSubstrate);
        d.discover(CardType::ApexSpore);
        let ids = d.to_id_strings();
        let d2 = DiscoveryState::from_id_strings(&ids);
        assert!(d2.contains(CardType::BioSubstrate));
        assert!(d2.contains(CardType::ApexSpore));
        assert_eq!(d.count(), d2.count());
    }
}
