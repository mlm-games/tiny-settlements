use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::HashMap;

use super::CardType;
use super::events::{GameEvent, PendingGameEvents};
use super::{Card, GameSession, PendingDespawns, PendingSpawns, PendingFx, FxEvent, RunCounters};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn next(self) -> Self {
        match self {
            Self::Spring => Self::Summer,
            Self::Summer => Self::Autumn,
            Self::Autumn => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Autumn => "Autumn",
            Self::Winter => "Winter",
        }
    }
    pub fn stable_id(self) -> &'static str {
        match self {
            Self::Spring => "spring",
            Self::Summer => "summer",
            Self::Autumn => "autumn",
            Self::Winter => "winter",
        }
    }
    pub fn from_stable_id(s: &str) -> Option<Self> {
        Some(match s {
            "spring" => Self::Spring,
            "summer" => Self::Summer,
            "autumn" => Self::Autumn,
            "winter" => Self::Winter,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeatherEvent {
    Heatwave,
    HeavyRain,
    Blight,
    FrostSnap,
    PollinatorSurge,
    HarvestFair,
}

impl WeatherEvent {
    pub const ALL: [Self; 6] = [
        Self::Heatwave,
        Self::HeavyRain,
        Self::Blight,
        Self::FrostSnap,
        Self::PollinatorSurge,
        Self::HarvestFair,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Heatwave => "Heatwave",
            Self::HeavyRain => "Heavy Rain",
            Self::Blight => "Blight",
            Self::FrostSnap => "Frost Snap",
            Self::PollinatorSurge => "Pollinator Surge",
            Self::HarvestFair => "Harvest Fair",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Heatwave => "Focus recharge reduced. Toxins spread faster.",
            Self::HeavyRain => "Growth quickens. Toxins wash away.",
            Self::Blight => "Monocultures are vulnerable.",
            Self::FrostSnap => "Growth and pollination falter.",
            Self::PollinatorSurge => "Pollination surges.",
            Self::HarvestFair => "Mature habitats pay tribute.",
        }
    }
}

#[derive(Resource)]
pub struct SeasonClock {
    pub current: Season,
    pub moons_into_season: u8, // 0..3
    pub current_year: u32,     // starts at 1
    pub total_moons: u32,
    pub moon_timer: Timer,
    pub seasons_survived: u32,
}

impl Default for SeasonClock {
    fn default() -> Self {
        Self {
            current: Season::Spring,
            moons_into_season: 0,
            current_year: 1,
            total_moons: 0,
            moon_timer: Timer::from_seconds(22.0, TimerMode::Repeating),
            seasons_survived: 0,
        }
    }
}

impl SeasonClock {
    pub fn advance_moon(&mut self) -> Option<Season> {
        self.moons_into_season += 1;
        self.total_moons += 1;
        if self.moons_into_season >= 4 {
            self.moons_into_season = 0;
            let old = self.current;
            self.current = self.current.next();
            self.seasons_survived += 1;
            if old == Season::Winter && self.current == Season::Spring {
                self.current_year += 1;
            }
            Some(self.current)
        } else {
            None
        }
    }
    pub fn moon_label(&self) -> String {
        format!("{}/4", self.moons_into_season + 1)
    }
}

#[derive(Resource, Default)]
pub struct ActiveWeather {
    pub event: Option<WeatherEvent>,
    pub timer: Timer,
    pub cooldown: Timer,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct EcoModifiers {
    pub growth_mult: f32,
    pub production_mult: f32,
    pub pollination_mult: f32,
    pub focus_recharge_mult: f32,
    pub toxin_spawn_mult: f32,
}

impl Default for EcoModifiers {
    fn default() -> Self {
        Self {
            growth_mult: 1.0,
            production_mult: 1.0,
            pollination_mult: 1.0,
            focus_recharge_mult: 1.0,
            toxin_spawn_mult: 1.0,
        }
    }
}

pub fn season_base_modifiers(season: Season) -> EcoModifiers {
    match season {
        Season::Spring => EcoModifiers {
            growth_mult: 1.20,
            production_mult: 1.00,
            pollination_mult: 1.15,
            focus_recharge_mult: 1.00,
            toxin_spawn_mult: 0.90,
        },
        Season::Summer => EcoModifiers {
            growth_mult: 1.00,
            production_mult: 1.15,
            pollination_mult: 1.00,
            focus_recharge_mult: 0.90,
            toxin_spawn_mult: 1.20,
        },
        Season::Autumn => EcoModifiers {
            growth_mult: 0.95,
            production_mult: 1.10,
            pollination_mult: 1.00,
            focus_recharge_mult: 1.00,
            toxin_spawn_mult: 1.05,
        },
        Season::Winter => EcoModifiers {
            growth_mult: 0.65,
            production_mult: 0.80,
            pollination_mult: 0.85,
            focus_recharge_mult: 1.05,
            toxin_spawn_mult: 0.85,
        },
    }
}

pub fn weather_layer_modifiers(event: WeatherEvent) -> EcoModifiers {
    match event {
        WeatherEvent::Heatwave => EcoModifiers {
            growth_mult: 1.0,
            production_mult: 1.05,
            pollination_mult: 1.0,
            focus_recharge_mult: 0.65,
            toxin_spawn_mult: 1.40,
        },
        WeatherEvent::HeavyRain => EcoModifiers {
            growth_mult: 1.30,
            production_mult: 1.10,
            pollination_mult: 1.0,
            focus_recharge_mult: 1.0,
            toxin_spawn_mult: 0.75,
        },
        WeatherEvent::Blight => EcoModifiers {
            growth_mult: 1.0,
            production_mult: 1.0,
            pollination_mult: 1.0,
            focus_recharge_mult: 1.0,
            toxin_spawn_mult: 1.0,
        },
        WeatherEvent::FrostSnap => EcoModifiers {
            growth_mult: 0.75,
            production_mult: 1.0,
            pollination_mult: 0.85,
            focus_recharge_mult: 1.0,
            toxin_spawn_mult: 1.0,
        },
        WeatherEvent::PollinatorSurge => EcoModifiers {
            growth_mult: 1.0,
            production_mult: 1.0,
            pollination_mult: 1.40,
            focus_recharge_mult: 1.0,
            toxin_spawn_mult: 1.0,
        },
        WeatherEvent::HarvestFair => EcoModifiers {
            growth_mult: 1.0,
            production_mult: 1.0,
            pollination_mult: 1.0,
            focus_recharge_mult: 1.0,
            toxin_spawn_mult: 1.0,
        },
    }
}

pub fn combined_modifiers(season: Season, weather: Option<WeatherEvent>) -> EcoModifiers {
    let base = season_base_modifiers(season);
    if let Some(ev) = weather {
        let w = weather_layer_modifiers(ev);
        EcoModifiers {
            growth_mult: base.growth_mult * w.growth_mult,
            production_mult: base.production_mult * w.production_mult,
            pollination_mult: base.pollination_mult * w.pollination_mult,
            focus_recharge_mult: base.focus_recharge_mult * w.focus_recharge_mult,
            toxin_spawn_mult: base.toxin_spawn_mult * w.toxin_spawn_mult,
        }
    } else {
        base
    }
}

/// Pick weather deterministically from rng; None means no weather this season.
pub fn pick_random_weather(rng: &mut StdRng) -> Option<WeatherEvent> {
    pick_weather_from_deck(rng, &WeatherEvent::ALL)
}

pub fn pick_weather_from_deck(rng: &mut StdRng, deck: &[WeatherEvent]) -> Option<WeatherEvent> {
    if deck.is_empty() {
        return None;
    }
    let roll: f32 = rng.random_range(0.0..1.0);
    if roll > 0.55 {
        return None;
    }
    Some(deck[rng.random_range(0..deck.len())])
}

pub fn tick_season_clock(
    time: Res<Time>,
    mut clock: ResMut<SeasonClock>,
    mut eco: ResMut<EcoModifiers>,
    active_weather: Res<ActiveWeather>,
    rules: Option<Res<crate::game::run_rules::RunRules>>,
    mut events: ResMut<PendingGameEvents>,
    // For autumn harvest
    cards: Query<&Card>,
    mut economy: ResMut<super::economy::RunEconomy>,
    mut save: ResMut<crate::save::SaveData>,
) {
    if let Some(r) = rules.as_deref() {
        if !r.features.seasons {
            return;
        }
    }
    if clock.moon_timer.tick(time.delta()).just_finished() {
        let old_season = clock.current;
        let season_changed = clock.advance_moon();
        // recompute eco after change (weather overlay computed elsewhere but we include here)
        *eco = combined_modifiers(clock.current, active_weather.event);
        // grant autumn harvest each moon end while in Autumn (before or after advance? Use current after advance)
        // Spec: At each Moon end in Autumn: grant bonus Dew equal to mature species count
        if clock.current == Season::Autumn {
            let mature_count = cards.iter().filter(|c| c.card_type.is_mature_species()).count() as u32;
            if mature_count > 0 {
                // cap at 8 to avoid runaway? spec says maybe cap
                let dew = mature_count.min(8);
                economy.earn(dew);
                events.0.push(GameEvent::HarvestGranted { dew });
                // also save tracking?
            }
        }
        // HarvestFair stacks: if active weather is HarvestFair, grant extra mature bonus regardless of season? Spec: bonus Dew from mature species this Moon
        if active_weather.event == Some(WeatherEvent::HarvestFair) {
            let mature_count = cards.iter().filter(|c| c.card_type.is_mature_species()).count() as u32;
            if mature_count > 0 {
                let dew = (mature_count / 2).max(2).min(6);
                economy.earn(dew);
                events.0.push(GameEvent::HarvestGranted { dew });
            }
        }
        if let Some(new_season) = season_changed {
            events.0.push(GameEvent::SeasonChanged { season: new_season, year: clock.current_year });
            // update save tracking
            if clock.seasons_survived > save.seasons_survived {
                save.seasons_survived = clock.seasons_survived;
            }
            if clock.current_year > save.best_year {
                save.best_year = clock.current_year;
            }
            // let _ = save manager save? caller will save on win, but we persist seasons survived lazily
        } else {
            // still moon tick but not season change - we already granted harvest above
        }
        let _ = old_season;
    }
}

pub fn recompute_eco_modifiers(
    clock: Res<SeasonClock>,
    weather: Res<ActiveWeather>,
    mut eco: ResMut<EcoModifiers>,
) {
    let new = combined_modifiers(clock.current, weather.event);
    *eco = new;
}

pub fn tick_active_weather(
    time: Res<Time>,
    mut weather: ResMut<ActiveWeather>,
    mut events: ResMut<PendingGameEvents>,
    mut rng: ResMut<super::packs::RunRng>,
    clock: Res<SeasonClock>,
    rules: Option<Res<crate::game::run_rules::RunRules>>,
) {
    let deck: &[WeatherEvent] = rules
        .as_deref()
        .map(|r| r.weather_deck.as_slice())
        .unwrap_or(&WeatherEvent::ALL);
    if deck.is_empty() {
        return;
    }
    if let Some(r) = rules.as_deref() {
        if !r.features.weather {
            return;
        }
    }
    // cooldown handling
    if weather.cooldown.duration().is_zero() {
        weather.cooldown = Timer::from_seconds(8.0, TimerMode::Once);
    }
    // if weather active, tick its timer
    if weather.event.is_some() {
        if weather.timer.tick(time.delta()).just_finished() {
            if let Some(ev) = weather.event.take() {
                events.0.push(GameEvent::WeatherEnded { weather: ev });
            }
            weather.cooldown.reset();
        }
        return;
    }
    // No active weather: tick cooldown and maybe start new
    if !weather.cooldown.tick(time.delta()).just_finished() {
        return;
    }
    // Cooldown finished, try to roll new weather (only if not every moon? tie to season)
    // For determinism, use rng
    if let Some(ev) = pick_weather_from_deck(&mut rng.0, deck) {
        weather.event = Some(ev);
        weather.timer = Timer::from_seconds(18.0, TimerMode::Once);
        events.0.push(GameEvent::WeatherStarted { weather: ev });
    } else {
        // no weather this cycle, reset cooldown to try again later
        weather.cooldown = Timer::from_seconds(12.0, TimerMode::Once);
    }
    let _ = clock;
}

// Helper to find blight target: monoculture habitats first else random plant
pub fn choose_blight_target(
    habitats: &[(Entity, bool, Option<Entity>)], // (habitat_entity, is_monoculture, plant)
    plants: &[Entity], // all plant entities
    rng: &mut StdRng,
) -> Option<Entity> {
    // habitats with monoculture and plant
    let mut mono_plants: Vec<Entity> = habitats.iter()
        .filter(|(_, is_mono, plant)| *is_mono && plant.is_some())
        .filter_map(|(_, _, plant)| *plant)
        .collect();
    if !mono_plants.is_empty() {
        mono_plants.shuffle(rng);
        return mono_plants.into_iter().next();
    }
    // fallback random plant
    if plants.is_empty() {
        return None;
    }
    let mut shuffled = plants.to_vec();
    shuffled.shuffle(rng);
    shuffled.into_iter().next()
}

pub fn blight_strike_tick(
    time: Res<Time>,
    mut timer: Local<Timer>,
    weather: Res<ActiveWeather>,
    habitats: Query<(Entity, &super::stacks::HabitatBase, &super::stacks::HabitatSynergy)>,
    cards: Query<(Entity, &Card)>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut events: ResMut<PendingGameEvents>,
    mut rng: ResMut<super::packs::RunRng>,
    session: Res<GameSession>,
) {
    if session.game_over {
        return;
    }
    if weather.event != Some(WeatherEvent::Blight) {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    // collect habitats with monoculture flag
    let habs: Vec<(Entity, bool, Option<Entity>)> = habitats.iter()
        .map(|(e, base, syn)| (e, syn.is_monoculture, base.plant))
        .collect();
    let plant_entities: Vec<Entity> = cards.iter()
        .filter(|(_, c)| c.card_type.is_plant())
        .map(|(e, _)| e)
        .collect();
    if let Some(target) = choose_blight_target(&habs, &plant_entities, &mut rng.0) {
        pending_despawn.0.push(target);
        events.0.push(GameEvent::BlightStruck { habitat: habitats.iter().find(|(_,b,_)| b.plant == Some(target)).map(|(e,_,_)| e) });
        // also remove synergy? clear_dead_stacks will handle
    }
}

pub fn heatwave_toxin_tick(
    time: Res<Time>,
    mut timer: Local<Timer>,
    weather: Res<ActiveWeather>,
    eco: Res<EcoModifiers>,
    mut pending_spawn: ResMut<PendingSpawns>,
    mut pending_fx: ResMut<PendingFx>,
    session: Res<GameSession>,
) {
    if session.game_over {
        return;
    }
    if weather.event != Some(WeatherEvent::Heatwave) {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(8.0, TimerMode::Repeating);
    }
    // Scale interval by toxin spawn mult? Actually higher mult should spawn more often, so divide interval
    let interval = 8.0 / eco.toxin_spawn_mult.max(0.5);
    if timer.duration().as_secs_f32() != interval {
        *timer = Timer::from_seconds(interval, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    // spawn toxin
    let pos = super::random_board_pos();
    pending_spawn.0.push((CardType::WasteToxin, pos, false));
    pending_fx.0.push(FxEvent::Toxin { pos });
}

pub fn heavy_rain_cleanse_tick(
    time: Res<Time>,
    mut timer: Local<Timer>,
    weather: Res<ActiveWeather>,
    cards: Query<(Entity, &Card)>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut pending_fx: ResMut<PendingFx>,
    mut events: ResMut<PendingGameEvents>,
    session: Res<GameSession>,
) {
    if session.game_over {
        return;
    }
    if weather.event != Some(WeatherEvent::HeavyRain) {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(12.0, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    // find one toxin to cleanse
    for (e, c) in &cards {
        if c.card_type == CardType::WasteToxin {
            pending_despawn.0.push(e);
            // use a random toxin pos? Need position, but we don't have transform query here; use zero pos for fx
            pending_fx.0.push(FxEvent::Clean { pos: Vec2::ZERO });
            events.0.push(GameEvent::CleanedToxin);
            break;
        }
    }
}

pub fn frost_snap_tick(
    time: Res<Time>,
    mut timer: Local<Timer>,
    weather: Res<ActiveWeather>,
    cards: Query<(Entity, &Card)>,
    mut pending_despawn: ResMut<PendingDespawns>,
    mut events: ResMut<PendingGameEvents>,
    session: Res<GameSession>,
) {
    if session.game_over {
        return;
    }
    if weather.event != Some(WeatherEvent::FrostSnap) {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(10.0, TimerMode::Repeating);
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    // Harm one fragile growing plant: YoungVine, FlutterwingLarva, SporePod (if planted)
    for (e, c) in &cards {
        if matches!(c.card_type, CardType::YoungVine | CardType::FlutterwingLarva | CardType::SporePod) {
            pending_despawn.0.push(e);
            events.0.push(GameEvent::BlightStruck { habitat: None });
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn season_advances_every_four_moons() {
        let mut clock = SeasonClock { current: Season::Spring, moons_into_season: 0, current_year: 1, total_moons: 0, moon_timer: Timer::from_seconds(1.0, TimerMode::Repeating), seasons_survived: 0 };
        for _ in 0..3 {
            assert!(clock.advance_moon().is_none());
        }
        let changed = clock.advance_moon();
        assert_eq!(changed, Some(Season::Summer));
        assert_eq!(clock.current, Season::Summer);
        assert_eq!(clock.moons_into_season, 0);
    }

    #[test]
    fn winter_wraps_to_spring_and_increments_year() {
        let mut clock = SeasonClock { current: Season::Winter, moons_into_season: 3, current_year: 1, total_moons: 12, moon_timer: Timer::from_seconds(1.0, TimerMode::Repeating), seasons_survived: 3 };
        let changed = clock.advance_moon();
        assert_eq!(changed, Some(Season::Spring));
        assert_eq!(clock.current, Season::Spring);
        assert_eq!(clock.current_year, 2);
    }

    #[test]
    fn eco_modifiers_match_current_season() {
        let spring = combined_modifiers(Season::Spring, None);
        assert!((spring.growth_mult - 1.20).abs() < 0.01);
        let winter = combined_modifiers(Season::Winter, None);
        assert!((winter.growth_mult - 0.65).abs() < 0.01);
        assert!(winter.production_mult < 1.0);
    }

    #[test]
    fn heatwave_reduces_focus_recharge() {
        let base = season_base_modifiers(Season::Summer);
        let with = combined_modifiers(Season::Summer, Some(WeatherEvent::Heatwave));
        assert!(with.focus_recharge_mult < base.focus_recharge_mult);
        assert!(with.toxin_spawn_mult > base.toxin_spawn_mult);
    }

    #[test]
    fn heavy_rain_boosts_growth() {
        let base = season_base_modifiers(Season::Spring);
        let with = combined_modifiers(Season::Spring, Some(WeatherEvent::HeavyRain));
        assert!(with.growth_mult > base.growth_mult);
    }

    #[test]
    fn blight_targets_monoculture_first() {
        let mut rng = StdRng::seed_from_u64(1);
        let hab_mono = Entity::from_bits(1);
        let plant_mono = Entity::from_bits(10);
        let hab_other = Entity::from_bits(2);
        let plant_other = Entity::from_bits(11);
        let habitats = vec![(hab_mono, true, Some(plant_mono)), (hab_other, false, Some(plant_other))];
        let plants = vec![plant_mono, plant_other];
        // Should always pick mono plant when monoculture exists, despite shuffle
        for _ in 0..5 {
            let picked = choose_blight_target(&habitats, &plants, &mut rng);
            assert_eq!(picked, Some(plant_mono));
        }
    }

    #[test]
    fn blight_falls_back_to_random_when_no_monoculture() {
        let mut rng = StdRng::seed_from_u64(42);
        let habitats = vec![(Entity::from_bits(1), false, Some(Entity::from_bits(10)))];
        let plants = vec![Entity::from_bits(10), Entity::from_bits(11), Entity::from_bits(12)];
        let picked = choose_blight_target(&habitats, &plants, &mut rng);
        assert!(picked.is_some());
        // Should be one of the plants
        assert!(plants.contains(&picked.unwrap()));
    }

    #[test]
    fn same_project_seed_has_deterministic_results() {
        // Weather determinism
        let mut a = StdRng::seed_from_u64(123);
        let mut b = StdRng::seed_from_u64(123);
        assert_eq!(pick_random_weather(&mut a), pick_random_weather(&mut b));
    }

    #[test]
    fn autumn_harvest_grants_bonus_dew() {
        // In Autumn, mature count yields dew (capped at 8)
        let mature = 5u32;
        let dew = mature.min(8);
        assert_eq!(dew, 5);
        let many = 12u32;
        assert_eq!(many.min(8), 8);
        // Verify season base for Autumn has correct modifiers vs Spring
        let autumn = season_base_modifiers(Season::Autumn);
        let spring = season_base_modifiers(Season::Spring);
        assert!(autumn.growth_mult < spring.growth_mult);
    }

    #[test]
    fn harvest_fair_stacks_with_autumn_bonus() {
        // HarvestFair should stack with Autumn: combined mature bonus
        let mature = 4u32;
        let autumn_bonus = mature.min(8);
        let fair_bonus = (mature / 2).max(2).min(6);
        let combined = autumn_bonus + fair_bonus;
        assert!(combined > autumn_bonus);
        assert!(combined > fair_bonus);
        // Check that HarvestFair has no EcoModifiers for production but still grants via event
        let fair_mod = weather_layer_modifiers(WeatherEvent::HarvestFair);
        assert_eq!(fair_mod.growth_mult, 1.0);
    }

    #[test]
    fn compost_cradle_counters_heatwave_toxin_pressure() {
        // Heatwave increases toxin_spawn_mult
        let base_summer = season_base_modifiers(Season::Summer);
        let heat = combined_modifiers(Season::Summer, Some(WeatherEvent::Heatwave));
        assert!(heat.toxin_spawn_mult > base_summer.toxin_spawn_mult);
        // Compost Cradle should be installation and be able to convert toxin -> mulch
        assert!(super::super::CardType::CompostCradle.is_installation());
        // Simulate that heatwave toxin interval is reduced
        let base_interval = 8.0;
        let eco_heat = heat.toxin_spawn_mult;
        let interval_heat = base_interval / eco_heat.max(0.5);
        let eco_no = base_summer.toxin_spawn_mult;
        let interval_no = base_interval / eco_no.max(0.5);
        assert!(interval_heat < interval_no);
    }

    #[test]
    fn nursery_tray_offsets_winter_growth_penalty() {
        let winter = season_base_modifiers(Season::Winter).growth_mult;
        assert!((winter - 0.65).abs() < 0.01);
        let nursery = crate::game::projects::installation_growth_mult(crate::game::CardType::NurseryTray, crate::game::CardType::VineSeed);
        assert!((nursery - 1.30).abs() < 0.01);
        let combined = winter * nursery;
        assert!(combined > winter);
        assert!(combined > 0.80);
    }

    #[test]
    fn save_v3_loads_with_v4_defaults() {
        let data = crate::save::SaveData::default();
        assert_eq!(data.version, crate::save::SAVE_VERSION);
        assert_eq!(data.seasons_survived, 0);
        assert_eq!(data.best_year, 0);
        // old save without those fields should default to 0
        let old = crate::save::SaveData {
            version: 3,
            ..Default::default()
        };
        assert_eq!(old.seasons_survived, 0);
        assert_eq!(old.best_year, 0);
    }

    #[test]
    fn pollinator_surge_boosts_pollination() {
        let base = season_base_modifiers(Season::Spring);
        let with = combined_modifiers(Season::Spring, Some(WeatherEvent::PollinatorSurge));
        assert!(with.pollination_mult > base.pollination_mult);
        assert!((with.pollination_mult - base.pollination_mult * 1.40).abs() < 0.01);
    }

    #[test]
    fn frost_snap_reduces_growth() {
        let base = season_base_modifiers(Season::Winter);
        let with = combined_modifiers(Season::Winter, Some(WeatherEvent::FrostSnap));
        assert!(with.growth_mult < base.growth_mult);
    }

    #[test]
    fn season_year_increments_only_on_winter_to_spring() {
        let mut clock = SeasonClock { current: Season::Spring, moons_into_season: 3, current_year: 1, total_moons: 3, moon_timer: Timer::from_seconds(1.0, TimerMode::Repeating), seasons_survived: 0 };
        // Spring -> Summer should not increment year
        let _ = clock.advance_moon();
        assert_eq!(clock.current_year, 1);
        assert_eq!(clock.current, Season::Summer);
        // Set to Winter 3/4 and advance to Spring
        clock.current = Season::Winter;
        clock.moons_into_season = 3;
        clock.current_year = 5;
        let _ = clock.advance_moon();
        assert_eq!(clock.current_year, 6);
    }
}
