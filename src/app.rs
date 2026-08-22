use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use repose_bevy::{ReposePlugin, ReposePluginSettings};
use repose_core::{prelude::Modifier, remember};
use repose_ui::overlay::OverlayHandle;

use crate::asset_tracking::AssetsLoading;
use crate::dev_tools::DevToolsPlugin;
use crate::game::{GamePlugin, RestartFlag};
use crate::menus::{self, UiAction, UiBridge};
use crate::save::SaveData;
use crate::screens::ScreensPlugin;
use crate::theme::ThemePlugin;
use game_utils_bevy::{
    EcosystemPlugin,
    audio::{AudioChannels, MusicChannel},
    i18n::{self, I18nPlugin, LocaleResources},
    post_process::{ScreenEffectSettings, sync_post_process_settings},
    save::{SaveManager, SavePlugin},
    screen_effects::CameraBase,
    time_scale::TimeScaleControl,
    transitions::Transition,
};

const TRANSLATION_KEYS: &[&str] = &[
    "app-title",
    "start-game",
    "settings",
    "credits",
    "quit",
    "paused",
    "resume",
    "quit-to-title",
    "save",
    "back",
    "master-volume",
    "sfx-volume",
    "music-volume",
    "language",
    "biodiversity",
    "toxins",
    "toxin-warning",
    "wins",
    "focus",
    "controls-hint",
    "loading",
    "restart-hint",
    "you-win",
    "you-lose",
    // Phase 1
    "dew",
    "discoveries",
    "commissions",
    "satchels",
    "journal",
    "journal-open",
    "journal-close",
    "reward",
    "locked",
    "buy",
    "sold",
    "pack-soil",
    "pack-pollinator",
    "pack-symbiosis",
    "pack-draws",
    "need-discoveries",
    "need-commissions",
    "exchange-hint",
    "toast-discovered",
    "commission-complete",
    "empty-commissions",
    "not-enough-dew",
];

const LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../assets/locales/en/main.ftl")),
    ("es", include_str!("../assets/locales/es/main.ftl")),
    ("fr", include_str!("../assets/locales/fr/main.ftl")),
    ("de", include_str!("../assets/locales/de/main.ftl")),
    ("ja", include_str!("../assets/locales/ja/main.ftl")),
    ("zh", include_str!("../assets/locales/zh/main.ftl")),
    ("pt", include_str!("../assets/locales/pt/main.ftl")),
];

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Splash,
    Loading,
    Title,
    InGame,
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub struct Paused(pub bool);

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverlayMenu {
    #[default]
    None,
    Settings,
    Credits,
    Pause,
}

#[derive(Resource, Default)]
pub struct PendingUnpause(pub Option<Timer>);

#[derive(Clone, Debug, Default)]
pub struct CommissionHud {
    pub title: String,
    pub progress: u32,
    pub need: u32,
    pub reward: u32,
}

// Phase 1 UI DTOs (new, template-accurate)
#[derive(Clone, Debug, Default)]
pub struct CommissionUi {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub progress: u32,
    pub target: u32,
    pub reward_dew: u32,
    pub complete: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PackUi {
    pub id: String, // "soil_and_spore" | "pollinator" | "symbiosis"
    pub name: String,
    pub cost: u32,
    pub draws: u32,
    pub unlocked: bool,
    pub affordable: bool,
    pub locked_reason: String,
}

#[derive(Clone, Debug, Default)]
pub struct JournalEntryUi {
    pub id: String,
    pub name: String,
    pub discovered: bool,
    pub blurb: String,
}

// Keep PackHud as alias for backward compat with game/mod.rs (will be phased out)
#[derive(Clone, Debug)]
pub struct PackHud {
    pub id: crate::game::PackId,
    pub name: String,
    pub cost: u32,
    pub unlocked: bool,
    pub can_afford: bool,
}

#[derive(Resource, Clone)]
pub struct SharedUi {
    pub phase: AppState,
    pub paused: bool,
    pub loading_progress: f32,
    pub overlay: OverlayMenu,
    pub master_vol: f32,
    pub sfx_vol: f32,
    pub music_vol: f32,
    pub biodiversity: u32,
    pub toxins: u32,
    pub focus: f32,
    pub max_focus: f32,
    pub status_line: String,
    pub game_over: bool,
    pub victory: bool,
    pub end_reason: String,
    pub high_biodiversity: u32,
    pub wins: u32,
    pub times_played: u32,
    pub transition_alpha: f32,
    pub flash_alpha: f32,
    pub language: String,
    pub saved_language: String,
    pub available_languages: Vec<String>,
    pub translations: HashMap<String, String>,
    // Phase 1 run HUD (legacy fields kept for compat)
    pub dew: u32,
    pub discoveries: u32,
    pub total_discoveries: u32,
    pub total_commissions_completed: u32,
    pub commissions: Vec<CommissionHud>,
    pub packs: Vec<PackHud>,
    // Phase 1 deep UI (template-accurate)
    pub discoveries_total: u32,
    pub commissions_done_run: u32,
    pub toast: String,
    pub toast_timer: f32,
    pub show_journal: bool,
    pub commissions_ui: Vec<CommissionUi>,
    pub packs_ui: Vec<PackUi>,
    pub journal: Vec<JournalEntryUi>,
}

impl Default for SharedUi {
    fn default() -> Self {
        Self {
            phase: AppState::Splash,
            paused: false,
            loading_progress: 0.0,
            overlay: OverlayMenu::None,
            master_vol: 1.0,
            sfx_vol: 1.0,
            music_vol: 0.8,
            biodiversity: 0,
            toxins: 0,
            focus: 100.0,
            max_focus: 100.0,
            status_line: String::new(),
            game_over: false,
            victory: false,
            end_reason: String::new(),
            high_biodiversity: 0,
            wins: 0,
            times_played: 0,
            transition_alpha: 0.0,
            flash_alpha: 0.0,
            language: "en".to_string(),
            saved_language: "en".to_string(),
            available_languages: vec!["en".to_string()],
            translations: HashMap::new(),
            dew: 0,
            discoveries: 0,
            total_discoveries: 23,
            total_commissions_completed: 0,
            commissions: Vec::new(),
            packs: Vec::new(),
            discoveries_total: 24,
            commissions_done_run: 0,
            toast: String::new(),
            toast_timer: 0.0,
            show_journal: false,
            commissions_ui: Vec::new(),
            packs_ui: Vec::new(),
            journal: Vec::new(),
        }
    }
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        let shared = Arc::new(Mutex::new(SharedUi::default()));
        let actions = Arc::new(Mutex::new(Vec::<UiAction>::new()));
        let shared_ui = shared.clone();
        let actions_ui = actions.clone();

        app.init_state::<AppState>()
            .insert_resource(Paused(false))
            .insert_resource(OverlayMenu::None)
            .insert_resource(PendingUnpause(None))
            .insert_resource(UiBridge {
                shared: shared.clone(),
                actions: actions.clone(),
            })
            .add_plugins(ReposePlugin::with_settings(
                ReposePluginSettings {
                    clear_alpha: 0.0,
                    compose_every_frame: true,
                    msaa_samples: 1,
                    overlay: true,
                },
                move |_s, _c| {
                    let st = shared_ui.lock().unwrap().clone();
                    let acts = actions_ui.clone();
                    let overlay_rc = remember(OverlayHandle::new);
                    let overlay = (*overlay_rc).clone();
                    let root = menus::compose_root(overlay.clone(), st, acts);
                    overlay.host(Modifier::new().fill_max_size(), root)
                },
            ))
            .add_plugins((
                ThemePlugin,
                EcosystemPlugin::<AppState>::new(I18nPlugin::new(TRANSLATION_KEYS, LOCALES)),
                SavePlugin::<SaveData>::new(SaveManager::new(
                    "com",
                    "mlm-games",
                    "tiny-settlements",
                    "save.ron",
                    2,
                )),
                ScreensPlugin,
                GamePlugin,
                DevToolsPlugin,
            ))
            .init_resource::<MusicStarted>()
            .add_systems(OnEnter(AppState::Title), start_music)
            .add_systems(Startup, (setup_camera, crate::game::load_card_art))
            .add_systems(
                Update,
                (
                    apply_saved_settings,
                    sync_shared_ui,
                    tick_toast,
                    sync_post_process_settings::<AppState>,
                    process_ui_actions,
                    handle_pause_input,
                    tick_pending_unpause,
                    sync_virtual_time_with_pause,
                )
                    .chain(),
            );
    }
}

fn apply_saved_settings(save: Res<SaveData>, mut locale: ResMut<LocaleResources>) {
    if !save.is_added() && !save.is_changed() {
        return;
    }
    if locale
        .available
        .iter()
        .any(|l| l == &save.settings.language)
    {
        locale.set_locale(&save.settings.language);
    }
}

#[derive(Resource, Default)]
struct MusicStarted(bool);

fn start_music(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut started: ResMut<MusicStarted>,
) {
    if started.0 {
        return;
    }
    started.0 = true;
    commands.spawn((
        MusicChannel,
        AudioPlayer::new(assets.load("audio/music_loop.ogg")),
        PlaybackSettings::LOOP,
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
        CameraBase {
            translation: Vec3::new(0.0, 0.0, 1000.0),
            rotation: 0.0,
        },
        ScreenEffectSettings::default(),
    ));
}

fn sync_shared_ui(
    state: Res<State<AppState>>,
    paused: Res<Paused>,
    overlay: Res<OverlayMenu>,
    bridge: Res<UiBridge>,
    save: Res<SaveData>,
    transition: Res<Transition<AppState>>,
    flash: Res<game_utils_bevy::screen_effects::FlashWhite>,
    locale: Res<LocaleResources>,
    mut channels: ResMut<AudioChannels>,
    loading: Option<Res<AssetsLoading>>,
    asset_server: Res<AssetServer>,
    // Phase 1 optional resources (only present InGame, hence Option)
    economy: Option<Res<crate::game::RunEconomy>>,
    discovery: Option<Res<crate::game::DiscoveryState>>,
    commissions: Option<Res<crate::game::CommissionBoard>>,
) {
    let Ok(mut ui) = bridge.shared.lock() else {
        return;
    };
    ui.phase = state.get().clone();
    ui.paused = paused.0;
    ui.overlay = *overlay;
    ui.high_biodiversity = save.high_biodiversity;
    ui.wins = save.wins;
    ui.times_played = save.times_played;
    if *overlay != OverlayMenu::Settings {
        ui.master_vol = save.settings.master_volume;
        ui.sfx_vol = save.settings.sfx_volume;
        ui.music_vol = save.settings.music_volume;
    }
    ui.transition_alpha = transition.overlay_alpha;
    ui.flash_alpha = flash.amount;
    ui.language = locale.current.clone();
    ui.available_languages = locale.available.clone();
    ui.translations = i18n::get_current_translations(&locale);
    ui.loading_progress = match loading {
        Some(l) if !l.0.is_empty() => {
            l.0.iter()
                .filter(|h| asset_server.is_loaded_with_dependencies(h.id()))
                .count() as f32
                / l.0.len() as f32
        }
        _ => 1.0,
    };
    channels.master = save.settings.master_volume;
    channels.sfx = save.settings.sfx_volume;
    channels.music = save.settings.music_volume;

    // Phase 1 HUD snapshot
    if let Some(eco) = economy.as_deref() {
        ui.dew = eco.dew;
    }
    if let Some(disc) = discovery.as_deref() {
        ui.discoveries = disc.count() as u32;
        ui.total_discoveries = disc.total_possible();
        ui.discoveries_total = disc.total_possible();
        // Build journal entries (discovered first)
        let mut entries: Vec<JournalEntryUi> = Vec::new();
        for ctype in crate::game::DiscoveryState::all_types() {
            let discovered = disc.contains(ctype);
            entries.push(JournalEntryUi {
                id: ctype.stable_id().to_string(),
                name: ctype.label().to_string(),
                discovered,
                blurb: if discovered {
                    format!("Discovered {}", ctype.label())
                } else {
                    String::new()
                },
            });
        }
        // Sort discovered first
        entries.sort_by(|a, b| b.discovered.cmp(&a.discovered));
        ui.journal = entries;
    }
    if let Some(com) = commissions.as_deref() {
        ui.commissions_done_run = com.total_completed;
        ui.total_commissions_completed = com.total_completed.max(save.total_commissions_completed);
        // Map to both legacy and new UI DTOs
        ui.commissions = com
            .active
            .iter()
            .map(|c| CommissionHud {
                title: c.title.to_string(),
                progress: c.progress,
                need: c.need,
                reward: c.reward_dew,
            })
            .collect();
        ui.commissions_ui = com
            .active
            .iter()
            .map(|c| CommissionUi {
                id: c.template_id.to_string(),
                title: c.title.to_string(),
                detail: format!("{} {}/{}", c.title, c.progress, c.need),
                progress: c.progress,
                target: c.need,
                reward_dew: c.reward_dew,
                complete: c.completed,
            })
            .collect();
    }
    // Packs rows (use helper or inline)
    {
        let dew = economy.as_deref().map(|e| e.dew).unwrap_or(0);
        let disc = discovery.as_deref().map(|d| d.count() as u32).unwrap_or(0);
        let done = commissions
            .as_deref()
            .map(|c| c.total_completed)
            .unwrap_or(0)
            .max(save.total_commissions_completed);
        let mut packs_legacy: Vec<PackHud> = Vec::new();
        let mut packs_ui: Vec<PackUi> = Vec::new();
        for def in crate::game::PACKS {
            let unlocked =
                disc >= def.required_discoveries as u32 && done >= def.required_commissions as u32;
            let affordable = dew >= def.cost;
            let locked_reason = if disc < def.required_discoveries as u32 {
                format!("Discover {} more", def.required_discoveries as u32 - disc)
            } else if done < def.required_commissions as u32 {
                format!(
                    "Complete {} more commissions",
                    def.required_commissions as u32 - done
                )
            } else {
                String::new()
            };
            packs_legacy.push(PackHud {
                id: def.id,
                name: def.name.to_string(),
                cost: def.cost,
                unlocked,
                can_afford: affordable,
            });
            packs_ui.push(PackUi {
                id: crate::game::pack_id_to_str(def.id).to_string(),
                name: def.name.to_string(),
                cost: def.cost,
                draws: def.draws as u32,
                unlocked,
                affordable,
                locked_reason,
            });
        }
        ui.packs = packs_legacy;
        ui.packs_ui = packs_ui;
    }
}

fn tick_toast(real: Res<Time<Real>>, bridge: Res<UiBridge>) {
    let Ok(mut ui) = bridge.shared.lock() else {
        return;
    };
    if ui.toast_timer > 0.0 {
        ui.toast_timer = (ui.toast_timer - real.delta_secs()).max(0.0);
        if ui.toast_timer <= 0.0 {
            ui.toast.clear();
        }
    }
}

fn tick_pending_unpause(
    real: Res<Time<Real>>,
    mut pending: ResMut<PendingUnpause>,
    mut paused: ResMut<Paused>,
) {
    let Some(timer) = pending.0.as_mut() else {
        return;
    };
    if timer.tick(real.delta()).just_finished() {
        pending.0 = None;
        paused.0 = false;
    }
}

fn set_vol(bridge: &UiBridge, field: impl Fn(&mut SharedUi) -> &mut f32, v: f32) {
    if let Ok(mut ui) = bridge.shared.lock() {
        *field(&mut ui) = v.clamp(0.0, 1.0);
    }
}

fn process_ui_actions(
    bridge: Res<UiBridge>,
    mut paused: ResMut<Paused>,
    mut overlay: ResMut<OverlayMenu>,
    mut save: ResMut<SaveData>,
    mut exit: MessageWriter<AppExit>,
    mut transition: ResMut<Transition<AppState>>,
    manager: Res<SaveManager>,
    mut pending_unpause: ResMut<PendingUnpause>,
    mut locale: ResMut<LocaleResources>,
    mut restart: ResMut<RestartFlag>,
    mut pack_queue: ResMut<crate::game::PackPurchaseQueue>,
) {
    let Ok(mut q) = bridge.actions.lock() else {
        return;
    };
    for action in q.drain(..) {
        match action {
            UiAction::StartGame => {
                transition.begin_to_state(AppState::Loading);
            }
            UiAction::OpenSettings => {
                if let Ok(mut ui) = bridge.shared.lock() {
                    ui.saved_language = locale.current.clone();
                }
                *overlay = OverlayMenu::Settings;
            }
            UiAction::OpenCredits => *overlay = OverlayMenu::Credits,
            UiAction::CloseOverlay => {
                if *overlay == OverlayMenu::Settings
                    && let Ok(ui) = bridge.shared.lock()
                {
                    locale.set_locale(&ui.saved_language);
                }
                match *overlay {
                    OverlayMenu::Settings | OverlayMenu::Credits if paused.0 => {
                        *overlay = OverlayMenu::Pause;
                    }
                    OverlayMenu::Pause if paused.0 => {
                        *overlay = OverlayMenu::None;
                        pending_unpause.0 = Some(Timer::from_seconds(0.2, TimerMode::Once));
                    }
                    _ => {
                        *overlay = OverlayMenu::None;
                    }
                }
            }
            UiAction::Resume => {
                *overlay = OverlayMenu::None;
                pending_unpause.0 = Some(Timer::from_seconds(0.2, TimerMode::Once));
            }
            UiAction::Restart => {
                restart.0 = true;
                paused.0 = false;
                *overlay = OverlayMenu::None;
                pending_unpause.0 = None;
            }
            UiAction::QuitToTitle => {
                paused.0 = false;
                *overlay = OverlayMenu::None;
                pending_unpause.0 = None;
                transition.begin_to_state(AppState::Title);
            }
            UiAction::QuitApp => {
                exit.write(AppExit::Success);
            }
            UiAction::SetMasterVol(v) => set_vol(&bridge, |ui| &mut ui.master_vol, v),
            UiAction::SetSfxVol(v) => set_vol(&bridge, |ui| &mut ui.sfx_vol, v),
            UiAction::SetMusicVol(v) => set_vol(&bridge, |ui| &mut ui.music_vol, v),
            UiAction::SaveSettings => {
                if let Ok(ui) = bridge.shared.lock() {
                    save.settings.master_volume = ui.master_vol;
                    save.settings.sfx_volume = ui.sfx_vol;
                    save.settings.music_volume = ui.music_vol;
                    save.settings.language = locale.current.clone();
                }
                let _ = manager.save(&*save);
                if let Ok(mut ui) = bridge.shared.lock() {
                    ui.saved_language = locale.current.clone();
                }
                if paused.0 {
                    *overlay = OverlayMenu::Pause;
                } else {
                    *overlay = OverlayMenu::None;
                }
            }
            UiAction::NextLanguage => {
                let available = locale.available.clone();
                let current = locale.current.clone();
                let idx = available.iter().position(|l| *l == current).unwrap_or(0);
                let next = (idx + 1) % available.len();
                if let Some(next_locale) = available.get(next) {
                    locale.set_locale(next_locale);
                }
            }
            UiAction::SetLanguage(ref lang) => {
                if locale.available.contains(lang) {
                    locale.set_locale(lang);
                }
            }
            UiAction::BuyPack(ref id) => {
                if let Some(pack) = crate::game::pack_id_from_str(id) {
                    pack_queue.0.push(pack);
                }
            }
            UiAction::OpenJournal => {
                if let Ok(mut ui) = bridge.shared.lock() {
                    ui.show_journal = true;
                }
            }
            UiAction::CloseJournal => {
                if let Ok(mut ui) = bridge.shared.lock() {
                    ui.show_journal = false;
                }
            }
            UiAction::DismissToast => {
                if let Ok(mut ui) = bridge.shared.lock() {
                    ui.toast.clear();
                    ui.toast_timer = 0.0;
                }
            }
        }
    }
}

fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut paused: ResMut<Paused>,
    mut overlay: ResMut<OverlayMenu>,
    mut pending_unpause: ResMut<PendingUnpause>,
    transition: Res<Transition<AppState>>,
    bridge: Res<UiBridge>,
) {
    if *state.get() != AppState::InGame {
        return;
    }
    if transition.block_input {
        return;
    }
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    // Journal has priority over pause
    if let Ok(mut ui) = bridge.shared.try_lock() {
        if ui.show_journal {
            ui.show_journal = false;
            return;
        }
        if ui.toast_timer > 0.0 {
            ui.toast.clear();
            ui.toast_timer = 0.0;
            // don't consume Esc for pause if toast was showing? still close toast first
            return;
        }
    }
    match *overlay {
        OverlayMenu::None if !paused.0 => {
            paused.0 = true;
            *overlay = OverlayMenu::Pause;
            pending_unpause.0 = None;
        }
        OverlayMenu::Pause => {
            *overlay = OverlayMenu::None;
            pending_unpause.0 = Some(Timer::from_seconds(0.2, TimerMode::Once));
        }
        OverlayMenu::Settings | OverlayMenu::Credits => {
            if paused.0 {
                *overlay = OverlayMenu::Pause;
            } else {
                *overlay = OverlayMenu::None;
            }
        }
        _ => {}
    }
}

fn sync_virtual_time_with_pause(
    paused: Res<Paused>,
    mut ctrl: ResMut<TimeScaleControl>,
    #[cfg(feature = "physics")] mut rapier_config: Query<
        &mut bevy_rapier2d::plugin::RapierConfiguration,
    >,
) {
    ctrl.paused = paused.0;
    #[cfg(feature = "physics")]
    for mut config in &mut rapier_config {
        config.physics_pipeline_active = !paused.0;
    }
}
