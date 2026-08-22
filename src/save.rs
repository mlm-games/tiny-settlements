use bevy::prelude::*;
use game_utils::save::Versioned;
use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u32 = 2;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct SaveData {
    #[serde(default)]
    pub version: u32,
    pub high_biodiversity: u32,
    pub wins: u32,
    pub times_played: u32,
    pub settings: SettingsData,

    #[serde(default)]
    pub discovered_cards: Vec<String>,
    #[serde(default)]
    pub total_commissions_completed: u32,
    #[serde(default)]
    pub best_run_discoveries: u32,
    #[serde(default)]
    pub total_dew_earned: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SettingsData {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub language: String,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.8,
            language: "en".to_string(),
        }
    }
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            high_biodiversity: 0,
            wins: 0,
            times_played: 0,
            settings: SettingsData::default(),
            discovered_cards: Vec::new(),
            total_commissions_completed: 0,
            best_run_discoveries: 0,
            total_dew_earned: 0,
        }
    }
}

impl Versioned for SaveData {
    fn version(&self) -> u32 {
        self.version
    }

    fn set_version(&mut self, version: u32) {
        self.version = version;
    }
}
