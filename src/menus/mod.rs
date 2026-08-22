use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use repose_core::View;
use repose_core::prelude::{
    AlignItems, AlignSelf, AnimationSpec, Color as RColor, Easing, JustifyContent, Modifier,
    remember,
};
use repose_material::material3::{
    ButtonConfig, DropdownMenu, DropdownMenuConfig, DropdownMenuEntry, DropdownMenuItem,
    FilledTonalButton, MenuState,
};
use repose_ui::anim_ext::{
    AnimatedVisibility, AnimatedVisibilityConfig, EnterTransition, ExitTransition,
};
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Column, Row, Text as RText, TextStyle, ViewExt, ZStack};

use crate::app::{AppState, CommissionUi, JournalEntryUi, OverlayMenu, PackUi, SharedUi};

fn t(translations: &HashMap<String, String>, key: &str, fallback: &str) -> String {
    translations
        .get(key)
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

#[derive(Clone, Debug)]
pub enum UiAction {
    StartGame,
    OpenSettings,
    OpenCredits,
    CloseOverlay,
    Resume,
    Restart,
    QuitToTitle,
    QuitApp,
    SetMasterVol(f32),
    SetSfxVol(f32),
    SetMusicVol(f32),
    SaveSettings,
    NextLanguage,
    SetLanguage(String),
    BuyPack(String),
    OpenJournal,
    CloseJournal,
    DismissToast,
}

#[derive(bevy::prelude::Resource, Clone)]
pub struct UiBridge {
    pub shared: Arc<Mutex<SharedUi>>,
    pub actions: Arc<Mutex<Vec<UiAction>>>,
}

fn spacer(h: f32) -> View {
    Column(Modifier::new().height(h).width(1.0))
}

fn hspace(_w: f32) -> View {
    Column(Modifier::new().width(_w).height(1.0))
}

fn popup_anim_config(key: &str) -> AnimatedVisibilityConfig {
    AnimatedVisibilityConfig {
        key: key.into(),
        spec: AnimationSpec::tween(Duration::from_millis(200), Easing::EaseOut),
        enter: EnterTransition::ScaleIn { initial: 0.95 },
        exit: ExitTransition::ScaleOut { target: 0.95 },
    }
}

fn slide_toast_config(key: &str) -> AnimatedVisibilityConfig {
    AnimatedVisibilityConfig {
        key: key.into(),
        spec: AnimationSpec::tween(Duration::from_millis(220), Easing::EaseOut),
        enter: EnterTransition::ScaleIn { initial: 0.9 },
        exit: ExitTransition::ScaleOut { target: 0.9 },
    }
}

pub fn compose_root(
    overlay: OverlayHandle,
    st: SharedUi,
    actions: Arc<Mutex<Vec<UiAction>>>,
) -> View {
    let root = ZStack(Modifier::new().fill_max_size());
    let settings_view = settings_ui(overlay.clone(), &st, actions.clone());

    let content = match st.phase {
        AppState::Splash => splash_ui(&st),
        AppState::Loading => loading_ui(&st),
        AppState::Title => ZStack(Modifier::new().fill_max_size()).child((
            title_ui(&st, actions.clone()),
            AnimatedVisibility(
                st.overlay == OverlayMenu::Settings,
                settings_view.clone(),
                popup_anim_config("title_settings"),
            ),
            AnimatedVisibility(
                st.overlay == OverlayMenu::Credits,
                credits_ui(&st, actions.clone()),
                popup_anim_config("title_credits"),
            ),
        )),
        AppState::InGame => ZStack(Modifier::new().fill_max_size()).child((
            ingame_shell(&st, actions.clone()),
            AnimatedVisibility(
                st.game_over,
                game_over_ui(&st, actions.clone()),
                popup_anim_config("game_over"),
            ),
            AnimatedVisibility(
                st.overlay == OverlayMenu::Pause,
                pause_overlay(&st, actions.clone()),
                popup_anim_config("pause"),
            ),
            AnimatedVisibility(
                st.overlay == OverlayMenu::Settings,
                settings_view.clone(),
                popup_anim_config("ingame_settings"),
            ),
            AnimatedVisibility(
                st.overlay == OverlayMenu::Credits,
                credits_ui(&st, actions.clone()),
                popup_anim_config("ingame_credits"),
            ),
            AnimatedVisibility(
                st.show_journal,
                journal_overlay(&st, actions.clone()),
                popup_anim_config("journal"),
            ),
            AnimatedVisibility(
                st.toast_timer > 0.05 && !st.toast.is_empty(),
                toast_banner(&st, actions.clone()),
                slide_toast_config("toast"),
            ),
        )),
    };

    if st.transition_alpha > 0.001 || st.flash_alpha > 0.001 {
        let fade_a = (st.transition_alpha.clamp(0.0, 1.0) * 255.0) as u8;
        let flash_a = (st.flash_alpha.clamp(0.0, 1.0) * 255.0) as u8;
        root.child((
            content,
            Column(
                Modifier::new()
                    .fill_max_size()
                    .background(RColor::from_rgba(0, 0, 0, fade_a)),
            ),
            Column(
                Modifier::new()
                    .fill_max_size()
                    .background(RColor::from_rgba(flash_a, flash_a, flash_a, flash_a)),
            ),
        ))
    } else {
        root.child(content)
    }
}

// ── shared atoms ───────────────────────────────────────────────────────────

fn col(r: u8, g: u8, b: u8) -> RColor {
    RColor::from_rgba(r, g, b, 255)
}

fn cola(r: u8, g: u8, b: u8, a: u8) -> RColor {
    RColor::from_rgba(r, g, b, a)
}

fn push(actions: &Arc<Mutex<Vec<UiAction>>>, a: UiAction) {
    if let Ok(mut q) = actions.lock() {
        q.push(a);
    }
}

fn panel(width: f32, children: impl Into<View>) -> View {
    Column(
        Modifier::new()
            .width(width)
            .padding(16.0)
            .background(cola(12, 22, 16, 210))
            .clip_rounded(14.0)
            .border(1.5, col(70, 120, 85), 14.0),
    )
    .child(children.into())
}

fn chip(label: String, bg: RColor, fg: RColor) -> View {
    Column(
        Modifier::new()
            .padding(8.0)
            .background(bg)
            .clip_rounded(8.0),
    )
    .child(RText(label).size(13.0).color(fg))
}

fn mk_button(label: &str, _bg: RColor, on_click: impl Fn() + 'static) -> View {
    let label = label.to_string();
    FilledTonalButton(
        Modifier::new().width(260.0).height(48.0).margin(6.0),
        on_click,
        ButtonConfig::default(),
        move || RText(label.clone()).size(18.0),
    )
}

fn mk_button_sm(label: &str, on_click: impl Fn() + 'static) -> View {
    let label = label.to_string();
    FilledTonalButton(
        Modifier::new().width(48.0).height(40.0),
        on_click,
        ButtonConfig::default(),
        move || RText(label.clone()).size(18.0),
    )
}

fn mk_button_wide(label: String, enabled_look: bool, on_click: impl Fn() + 'static) -> View {
    let fg = if enabled_look {
        RColor::WHITE
    } else {
        col(140, 150, 140)
    };
    FilledTonalButton(
        Modifier::new().width(240.0).height(44.0).margin(4.0),
        on_click,
        ButtonConfig::default(),
        move || RText(label.clone()).size(15.0).color(fg),
    )
}

fn section_title(text: String) -> View {
    RText(text).size(14.0).color(col(170, 210, 170))
}

fn body(text: String, size: f32, c: RColor) -> View {
    RText(text).size(size).color(c)
}

fn progress_bar(frac: f32, width: f32, height: f32, fill: RColor, track: RColor) -> View {
    let f = frac.clamp(0.0, 1.0);
    Column(
        Modifier::new()
            .width(width)
            .height(height)
            .background(track)
            .clip_rounded(height * 0.5),
    )
    .child(Column(
        Modifier::new()
            .width((width * f).max(if f > 0.0 { 2.0 } else { 0.0 }))
            .height(height)
            .background(fill)
            .clip_rounded(height * 0.5)
            .align_self(AlignSelf::FLEX_START),
    ))
}

// ── splash / loading / title ───────────────────────────────────────────────

fn splash_ui(st: &SharedUi) -> View {
    let tr = &st.translations;
    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(col(8, 14, 10)),
    )
    .child(
        RText(t(tr, "app-title", "Tiny Settlements"))
            .size(48.0)
            .color(RColor::WHITE),
    )
    .child(spacer(8.0))
    .child(
        RText("Cultivate · Combine · Commission")
            .size(16.0)
            .color(col(140, 180, 150)),
    )
}

fn loading_ui(st: &SharedUi) -> View {
    let pct = st.loading_progress.clamp(0.0, 1.0);
    let tr = &st.translations;
    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(col(8, 14, 10)),
    )
    .child(
        RText(t(tr, "loading", "Loading..."))
            .size(28.0)
            .color(RColor::WHITE),
    )
    .child(spacer(16.0))
    .child(progress_bar(pct, 320.0, 12.0, col(96, 180, 130), col(30, 40, 32)))
    .child(spacer(10.0))
    .child(
        RText(format!("{:.0}%", pct * 100.0))
            .size(16.0)
            .color(col(180, 200, 180)),
    )
}

fn title_ui(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let a1 = actions.clone();
    let a2 = actions.clone();
    let a3 = actions.clone();
    let a4 = actions.clone();
    let tr = &st.translations;

    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(cola(6, 12, 9, 40)),
    )
    .child(
        Column(
            Modifier::new()
                .width(480.0)
                .padding(28.0)
                .background(cola(8, 16, 12, 200))
                .clip_rounded(18.0)
                .border(2.0, col(70, 120, 90), 18.0)
                .align_items(AlignItems::CENTER),
        )
        .child(RText(t(tr, "app-title", "Tiny Settlements")).size(44.0).color(RColor::WHITE))
        .child(spacer(6.0))
        .child(body(
            "A living terrarium engine-builder".into(),
            15.0,
            col(150, 190, 160),
        ))
        .child(spacer(14.0))
        .child(Row(Modifier::new().gap(8.0).align_items(AlignItems::CENTER)).child((
            chip(
                format!(
                    "{} {}",
                    t(tr, "biodiversity", "Biodiversity"),
                    st.high_biodiversity
                ),
                col(36, 78, 48),
                col(170, 230, 180),
            ),
            chip(
                format!("{} {}", t(tr, "wins", "Wins"), st.wins),
                col(40, 50, 70),
                col(190, 200, 220),
            ),
            chip(
                format!(
                    "{} {}",
                    t(tr, "discoveries", "Discoveries"),
                    st.discoveries.max(0)
                ),
                col(50, 60, 40),
                col(210, 220, 160),
            ),
        )))
        .child(spacer(22.0))
        .child(mk_button(
            &t(tr, "start-game", "Start Garden"),
            col(60, 130, 90),
            move || push(&a1, UiAction::StartGame),
        ))
        .child(mk_button(&t(tr, "settings", "Settings"), col(70, 70, 90), move || {
            push(&a2, UiAction::OpenSettings)
        }))
        .child(mk_button(&t(tr, "credits", "Credits"), col(70, 70, 90), move || {
            push(&a3, UiAction::OpenCredits)
        }))
        .child(mk_button(&t(tr, "quit", "Quit"), col(160, 60, 60), move || {
            push(&a4, UiAction::QuitApp)
        })),
    )
}

// ── pause / settings / credits ───────────────────────────────────────────

fn pause_overlay(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let a1 = actions.clone();
    let a2 = actions.clone();
    let a3 = actions.clone();
    let tr = &st.translations;

    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(cola(0, 0, 0, 180)),
    )
    .child(panel(
        340.0,
        Column(Modifier::new().align_items(AlignItems::CENTER)).child((
            RText(t(tr, "paused", "Paused"))
                .size(34.0)
                .color(RColor::WHITE),
            spacer(14.0),
            mk_button(&t(tr, "resume", "Resume"), col(60, 140, 90), move || {
                push(&a1, UiAction::Resume)
            }),
            mk_button(&t(tr, "settings", "Settings"), col(70, 70, 90), move || {
                push(&a2, UiAction::OpenSettings)
            }),
            mk_button(
                &t(tr, "quit-to-title", "Quit to Title"),
                col(180, 60, 60),
                move || push(&a3, UiAction::QuitToTitle),
            ),
        )),
    ))
}

fn settings_ui(overlay: OverlayHandle, st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let a_m_down = actions.clone();
    let a_m_up = actions.clone();
    let a_s_down = actions.clone();
    let a_s_up = actions.clone();
    let a_mu_down = actions.clone();
    let a_mu_up = actions.clone();
    let a_save = actions.clone();
    let a_back = actions.clone();
    let master = st.master_vol;
    let sfx = st.sfx_vol;
    let music = st.music_vol;
    let tr = &st.translations;
    let lang = st.language.clone();
    let langs = st.available_languages.clone();
    let overlay_clone = overlay.clone();
    let actions_clone = actions.clone();

    let menu_state: Rc<MenuState> = remember(MenuState::new);
    let lang_items: Vec<DropdownMenuEntry> = langs
        .iter()
        .map(|l| {
            let a = actions_clone.clone();
            let code = l.clone();
            let mut item = DropdownMenuItem::new(l.clone(), move || {
                push(&a, UiAction::SetLanguage(code.clone()))
            });
            if l == &lang {
                item = item.disabled();
            }
            DropdownMenuEntry::Item(item)
        })
        .collect();
    let menu_trigger = menu_state.clone();
    let lang_label = lang.clone();
    let trigger = FilledTonalButton(
        Modifier::new().width(100.0).height(40.0),
        move || menu_trigger.open(),
        ButtonConfig::default(),
        move || RText(lang_label.clone()).size(18.0),
    );
    let lang_dropdown = DropdownMenu(
        menu_state,
        overlay_clone,
        Modifier::new(),
        trigger,
        lang_items,
        DropdownMenuConfig {
            min_width: 100.0,
            ..Default::default()
        },
    );

    let inner = Column(
        Modifier::new()
            .width(380.0)
            .padding(24.0)
            .background(col(18, 26, 20))
            .clip_rounded(14.0)
            .align_items(AlignItems::CENTER),
    )
    .child(
        RText(t(tr, "settings", "Settings"))
            .size(32.0)
            .color(RColor::WHITE),
    )
    .child(spacer(12.0))
    .child(body(
        format!("{}: {:.0}%", t(tr, "master-volume", "Master"), master * 100.0),
        16.0,
        RColor::WHITE,
    ))
    .child(Row(Modifier::new().gap(8.0)).child((
        mk_button_sm("-", move || push(&a_m_down, UiAction::SetMasterVol(master - 0.1))),
        mk_button_sm("+", move || push(&a_m_up, UiAction::SetMasterVol(master + 0.1))),
    )))
    .child(spacer(8.0))
    .child(body(
        format!("{}: {:.0}%", t(tr, "sfx-volume", "SFX"), sfx * 100.0),
        16.0,
        RColor::WHITE,
    ))
    .child(Row(Modifier::new().gap(8.0)).child((
        mk_button_sm("-", move || push(&a_s_down, UiAction::SetSfxVol(sfx - 0.1))),
        mk_button_sm("+", move || push(&a_s_up, UiAction::SetSfxVol(sfx + 0.1))),
    )))
    .child(spacer(8.0))
    .child(body(
        format!("{}: {:.0}%", t(tr, "music-volume", "Music"), music * 100.0),
        16.0,
        RColor::WHITE,
    ))
    .child(Row(Modifier::new().gap(8.0)).child((
        mk_button_sm("-", move || push(&a_mu_down, UiAction::SetMusicVol(music - 0.1))),
        mk_button_sm("+", move || push(&a_mu_up, UiAction::SetMusicVol(music + 0.1))),
    )))
    .child(spacer(8.0))
    .child(body(
        format!("{}:", t(tr, "language", "Language")),
        16.0,
        RColor::WHITE,
    ))
    .child(lang_dropdown)
    .child(spacer(16.0))
    .child(mk_button(&t(tr, "save", "Save"), col(60, 120, 200), move || {
        push(&a_save, UiAction::SaveSettings)
    }))
    .child(mk_button(&t(tr, "back", "Back"), col(70, 70, 90), move || {
        push(&a_back, UiAction::CloseOverlay)
    }));

    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(cola(0, 0, 0, 180)),
    )
    .child(inner)
}

fn credits_ui(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let a = actions.clone();
    let tr = &st.translations;
    let inner = panel(
        420.0,
        Column(Modifier::new().align_items(AlignItems::CENTER))
            .child(RText(t(tr, "credits", "Credits")).size(32.0).color(RColor::WHITE))
            .child(spacer(12.0))
            .child(body("Tiny Settlements".into(), 16.0, RColor::WHITE))
            .child(body(
                "Godot original → Bevy + Repose (mlm-games)".into(),
                14.0,
                col(180, 200, 180),
            ))
            .child(body(
                "Cultivate the Genesis Bloom · Grow your terrarium economy".into(),
                14.0,
                col(160, 190, 160),
            ))
            .child(spacer(8.0))
            .child(body("Engine: Bevy    UI: Repose".into(), 14.0, col(150, 160, 150)))
            .child(spacer(16.0))
            .child(mk_button(&t(tr, "back", "Back"), col(70, 70, 90), move || {
                push(&a, UiAction::CloseOverlay)
            })),
    );

    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(cola(0, 0, 0, 180)),
    )
    .child(inner)
}

// ── IN-GAME SHELL (deep Phase 1 UI) ────────────────────────────────────────

fn ingame_shell(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    ZStack(Modifier::new().fill_max_size()).child((
        Column(
            Modifier::new()
                .fill_max_size()
                .padding(12.0)
                .align_items(AlignItems::FLEX_START)
                .justify_content(JustifyContent::FLEX_START),
        )
        .child(top_bar(st, actions.clone())),
        Column(
            Modifier::new()
                .fill_max_size()
                .padding(12.0)
                .align_items(AlignItems::FLEX_START)
                .justify_content(JustifyContent::CENTER),
        )
        .child((
            commissions_panel(st),
            spacer(10.0),
            habitat_panel(st),
        )),
        Column(
            Modifier::new()
                .fill_max_size()
                .padding(12.0)
                .align_items(AlignItems::FLEX_END)
                .justify_content(JustifyContent::CENTER),
        )
        .child(satchels_panel(st, actions.clone())),
        Column(
            Modifier::new()
                .fill_max_size()
                .padding(12.0)
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::FLEX_END),
        )
        .child(bottom_status(st)),
    ))
}

fn top_bar(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let tr = &st.translations;
    let focus_frac = if st.max_focus > 0.0 {
        (st.focus / st.max_focus).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let a_journal = actions.clone();

    Row(
        Modifier::new()
            .fill_max_width()
            .padding(10.0)
            .background(cola(8, 16, 12, 200))
            .clip_rounded(12.0)
            .border(1.0, col(60, 100, 70), 12.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN),
    )
    .child((
        Row(Modifier::new().gap(10.0).align_items(AlignItems::CENTER)).child((
            metric_chip(
                t(tr, "biodiversity", "Biodiversity"),
                st.biodiversity.to_string(),
                col(40, 90, 55),
            ),
            metric_chip(
                t(tr, "dew", "Dew"),
                st.dew.to_string(),
                col(40, 80, 110),
            ),
            metric_chip(
                t(tr, "discoveries", "Discoveries"),
                format!("{}/{}", st.discoveries, st.discoveries_total),
                col(90, 80, 40),
            ),
        )),
        Column(Modifier::new().align_items(AlignItems::CENTER)).child((
            body(
                format!(
                    "{}  {:.0}%",
                    t(tr, "focus", "Gardener Focus"),
                    focus_frac * 100.0
                ),
                13.0,
                col(190, 220, 180),
            ),
            spacer(4.0),
            progress_bar(focus_frac, 220.0, 10.0, col(110, 200, 120), col(30, 45, 34)),
        )),
        mk_button_wide(
            t(tr, "journal-open", "Journal"),
            true,
            move || push(&a_journal, UiAction::OpenJournal),
        ),
    ))
}

fn metric_chip(label: String, value: String, bg: RColor) -> View {
    Column(
        Modifier::new()
            .padding(8.0)
            .background(bg)
            .clip_rounded(10.0)
            .align_items(AlignItems::CENTER),
    )
    .child((
        body(label, 11.0, col(200, 220, 200)),
        body(value, 20.0, RColor::WHITE),
    ))
}

fn commissions_panel(st: &SharedUi) -> View {
    let tr = &st.translations;
    let mut rows: Vec<View> = Vec::new();
    rows.push(section_title(t(tr, "commissions", "Commissions")));
    rows.push(spacer(8.0));

    // Prefer new UI DTOs, fall back to legacy
    let use_new = !st.commissions_ui.is_empty();
    if use_new {
        if st.commissions_ui.is_empty() {
            rows.push(body(
                t(tr, "empty-commissions", "No active commissions"),
                13.0,
                col(150, 160, 150),
            ));
        } else {
            for c in st.commissions_ui.iter().take(3) {
                rows.push(commission_card_new(c));
                rows.push(spacer(8.0));
            }
        }
    } else if st.commissions.is_empty() {
        rows.push(body(
            t(tr, "empty-commissions", "No active commissions"),
            13.0,
            col(150, 160, 150),
        ));
    } else {
        for c in st.commissions.iter().take(3) {
            // legacy mapping
            let ui = CommissionUi {
                id: c.title.clone(),
                title: c.title.clone(),
                detail: format!("{} {}/{}", c.title, c.progress, c.need),
                progress: c.progress,
                target: c.need,
                reward_dew: c.reward,
                complete: c.progress >= c.need,
            };
            rows.push(commission_card_new(&ui));
            rows.push(spacer(8.0));
        }
    }

    rows.push(spacer(4.0));
    rows.push(body(
        format!("{}: {}", "Completed", st.commissions_done_run),
        12.0,
        col(140, 170, 140),
    ));

    panel(280.0, Column(Modifier::new()).child(rows))
}

fn commission_card(c: &CommissionUi) -> View {
    commission_card_new(c)
}

fn commission_card_new(c: &CommissionUi) -> View {
    let frac = if c.target > 0 {
        c.progress as f32 / c.target as f32
    } else {
        0.0
    };
    let border = if c.complete {
        col(180, 200, 90)
    } else {
        col(55, 90, 65)
    };

    Column(
        Modifier::new()
            .width(248.0)
            .padding(10.0)
            .background(cola(16, 28, 20, 230))
            .clip_rounded(10.0)
            .border(1.2, border, 10.0),
    )
    .child((
        Row(
            Modifier::new()
                .fill_max_width()
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .align_items(AlignItems::CENTER),
        )
        .child((
            body(c.title.clone(), 14.0, RColor::WHITE),
            chip(
                format!("+{} Dew", c.reward_dew),
                col(35, 70, 100),
                col(160, 210, 255),
            ),
        )),
        spacer(4.0),
        body(c.detail.clone(), 12.0, col(170, 190, 170)),
        spacer(6.0),
        progress_bar(frac, 220.0, 8.0, col(120, 190, 130), col(28, 40, 30)),
        spacer(4.0),
        body(
            format!("{}/{}", c.progress.min(c.target), c.target),
            12.0,
            col(190, 210, 190),
        ),
    ))
}

fn habitat_panel(st: &SharedUi) -> View {
    let tr = &st.translations;
    if st.habitats.is_empty() {
        return panel(
            280.0,
            Column(Modifier::new())
                .child(section_title(t(tr, "habitats", "Habitats")))
                .child(spacer(6.0))
                .child(body(
                    t(tr, "habitat-hint", "Drop Bio-Substrate on the faint grid to found a habitat"),
                    13.0,
                    col(150, 175, 155),
                )),
        );
    }

    let header = Row(Modifier::new().gap(10.0).align_items(AlignItems::CENTER)).child((
        body(
            format!("{} {}", t(tr, "habitats", "Habitats"), st.habitat_count),
            16.0,
            col(210, 230, 200),
        ),
        body(
            format!("{} +{:.0}%", t(tr, "resonance", "Resonance"), st.total_resonance * 100.0),
            13.0,
            col(140, 200, 150),
        ),
    ));

    let mut list = Column(Modifier::new().gap(6.0));
    for h in st.habitats.iter().filter(|h| h.plant.is_some()).take(6) {
        let plant = h.plant.as_deref().unwrap_or("—");
        let comp = h.companion.as_deref().unwrap_or("—");
        let mono = if h.is_monoculture { " ⚠ mono" } else { "" };
        let line1 = format!("{} → {} [{}]{}", h.substrate, plant, comp, mono);
        let line2 = if let Some(ref s) = h.synergy_name {
            format!("✦ {}  ×{:.1}", s, h.production_mult)
        } else {
            format!("×{:.1}  div {}", h.production_mult, h.diversity)
        };

        list = list.child(
            Column(
                Modifier::new()
                    .padding(10.0)
                    .background(col(26, 36, 28))
                    .clip_rounded(8.0),
            )
            .child((
                body(line1, 13.0, col(220, 235, 210)),
                body(
                    line2,
                    12.0,
                    if h.is_monoculture {
                        col(220, 150, 110)
                    } else {
                        col(160, 200, 150)
                    },
                ),
            )),
        );
    }
    // If no planted habitats but some empty habitats, show empty slot count
    if st.habitats.iter().filter(|h| h.plant.is_some()).count() == 0 {
        list = list.child(body(
            t(tr, "stack-plant-hint", "Plant on habitat"),
            13.0,
            col(160, 190, 160),
        ));
    }

    panel(
        280.0,
        Column(Modifier::new())
            .child(header)
            .child(spacer(6.0))
            .child(list),
    )
}

fn satchels_panel(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let tr = &st.translations;
    let mut rows: Vec<View> = Vec::new();
    rows.push(section_title(t(tr, "satchels", "Seed Satchels")));
    rows.push(spacer(8.0));

    let use_new = !st.packs_ui.is_empty();
    let packs: Vec<(String, String, u32, u32, bool, bool, String)> = if use_new {
        st.packs_ui
            .iter()
            .map(|p| {
                (
                    p.id.clone(),
                    p.name.clone(),
                    p.cost,
                    p.draws,
                    p.unlocked,
                    p.affordable,
                    p.locked_reason.clone(),
                )
            })
            .collect()
    } else if st.packs.is_empty() {
        // Fallback before packs sync exists
        vec![
            (
                "soil_and_spore".into(),
                t(tr, "pack-soil", "Soil & Spore"),
                4,
                2,
                true,
                st.dew >= 4,
                String::new(),
            ),
            (
                "pollinator".into(),
                t(tr, "pack-pollinator", "Pollinator"),
                9,
                2,
                st.discoveries >= 5,
                st.dew >= 9,
                if st.discoveries < 5 {
                    format!("Discover {} types", 5usize.saturating_sub(st.discoveries as usize))
                } else {
                    String::new()
                },
            ),
            (
                "symbiosis".into(),
                t(tr, "pack-symbiosis", "Symbiosis"),
                15,
                3,
                st.discoveries >= 10 && st.commissions_done_run >= 3,
                st.dew >= 15,
                if st.discoveries < 10 {
                    format!("Discover {} types", 10usize.saturating_sub(st.discoveries as usize))
                } else if st.commissions_done_run < 3 {
                    format!(
                        "Complete {} commissions",
                        3usize.saturating_sub(st.commissions_done_run as usize)
                    )
                } else {
                    String::new()
                },
            ),
        ]
    } else {
        st.packs
            .iter()
            .map(|p| {
                (
                    crate::game::pack_id_to_str(p.id).to_string(),
                    p.name.clone(),
                    p.cost,
                    2,
                    p.unlocked,
                    p.can_afford,
                    String::new(),
                )
            })
            .collect()
    };

    for (id, name, cost, draws, unlocked, affordable, locked_reason) in packs {
        let pack = PackUi {
            id: id.clone(),
            name,
            cost,
            draws,
            unlocked,
            affordable,
            locked_reason,
        };
        rows.push(pack_card(st, &pack, actions.clone()));
        rows.push(spacer(8.0));
    }

    panel(270.0, Column(Modifier::new().align_items(AlignItems::CENTER)).child(rows))
}

fn pack_card(st: &SharedUi, p: &PackUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let tr = &st.translations;
    let id = p.id.clone();
    let can_buy = p.unlocked && p.affordable && !st.game_over;
    let a = actions;

    let status = if !p.unlocked {
        if p.locked_reason.is_empty() {
            t(tr, "locked", "Locked")
        } else {
            p.locked_reason.clone()
        }
    } else if !p.affordable {
        t(tr, "not-enough-dew", "Not enough Dew")
    } else {
        format!("{} · {} Dew", t(tr, "buy", "Buy"), p.cost)
    };

    let border = if can_buy {
        col(90, 160, 120)
    } else if p.unlocked {
        col(100, 90, 50)
    } else {
        col(60, 60, 60)
    };

    Column(
        Modifier::new()
            .width(240.0)
            .padding(10.0)
            .background(cola(14, 24, 18, 235))
            .clip_rounded(10.0)
            .border(1.2, border, 10.0)
            .align_items(AlignItems::CENTER),
    )
    .child((
        body(p.name.clone(), 15.0, RColor::WHITE),
        spacer(2.0),
        body(format!("{} cards", p.draws), 12.0, col(160, 180, 160)),
        spacer(6.0),
        mk_button_wide(status, can_buy, move || {
            push(&a, UiAction::BuyPack(id.clone()))
        }),
    ))
}

fn bottom_status(st: &SharedUi) -> View {
    let tr = &st.translations;
    let line = if !st.status_line.is_empty() && !st.game_over {
        st.status_line.clone()
    } else {
        t(
            tr,
            "exchange-hint",
            "Seed Exchange — drop surplus cards on the right board zone to sell",
        )
    };

    Column(
        Modifier::new()
            .padding(10.0)
            .background(cola(8, 14, 10, 190))
            .clip_rounded(10.0)
            .align_items(AlignItems::CENTER),
    )
    .child((
        body(line, 14.0, col(220, 210, 150)),
        spacer(4.0),
        body(
            t(
                tr,
                "controls-hint",
                "Drag cards · Sell at Exchange · Esc pause · R restart",
            ),
            12.0,
            col(140, 160, 140),
        ),
    ))
}

fn toast_banner(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let a = actions;
    Column(
        Modifier::new()
            .fill_max_size()
            .padding(18.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::FLEX_START),
    )
    .child(
        Row(
            Modifier::new()
                .padding(12.0)
                .background(cola(30, 50, 35, 240))
                .clip_rounded(12.0)
                .border(1.5, col(140, 200, 120), 12.0)
                .align_items(AlignItems::CENTER)
                .gap(12.0),
        )
        .child((
            body(st.toast.clone(), 16.0, RColor::WHITE),
            mk_button_sm("OK", move || push(&a, UiAction::DismissToast)),
        )),
    )
}

fn journal_overlay(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let tr = &st.translations;
    let a_close = actions.clone();

    let mut entries: Vec<View> = Vec::new();
    entries.push(
        Row(
            Modifier::new()
                .fill_max_width()
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .align_items(AlignItems::CENTER),
        )
        .child((
            RText(t(tr, "journal", "Garden Journal"))
                .size(28.0)
                .color(RColor::WHITE),
            mk_button_wide(t(tr, "journal-close", "Close"), true, move || {
                push(&a_close, UiAction::CloseJournal)
            }),
        )),
    );
    entries.push(spacer(6.0));
    entries.push(body(
        format!(
            "{}: {} / {}",
            t(tr, "discoveries", "Discoveries"),
            st.discoveries,
            st.discoveries_total
        ),
        14.0,
        col(180, 210, 180),
    ));
    entries.push(spacer(12.0));

    if st.journal.is_empty() {
        for name in [
            "Gardener",
            "Bio-Substrate",
            "Spore Pod",
            "Nutrient slime",
            "Basic Fungi",
            "…",
        ] {
            entries.push(journal_row(&JournalEntryUi {
                id: name.into(),
                name: name.into(),
                discovered: matches!(
                    name,
                    "Gardener" | "Bio-Substrate" | "Spore Pod" | "Nutrient slime"
                ),
                blurb: if name == "…" {
                    "Grow, craft, and open satchels to fill the journal.".into()
                } else {
                    String::new()
                },
            }));
            entries.push(spacer(6.0));
        }
    } else {
        for e in &st.journal {
            entries.push(journal_row(e));
            entries.push(spacer(6.0));
        }
    }

    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(cola(0, 0, 0, 200)),
    )
    .child(
        Column(
            Modifier::new()
                .width(520.0)
                .padding(20.0)
                .background(col(14, 24, 18))
                .clip_rounded(16.0)
                .border(2.0, col(70, 120, 85), 16.0),
        )
        .child(entries),
    )
}

fn journal_row(e: &JournalEntryUi) -> View {
    let (title, fg, bg) = if e.discovered {
        (e.name.clone(), RColor::WHITE, cola(30, 50, 35, 255))
    } else {
        ("???".into(), col(120, 130, 120), cola(22, 28, 24, 255))
    };

    Column(
        Modifier::new()
            .width(480.0)
            .padding(10.0)
            .background(bg)
            .clip_rounded(8.0),
    )
    .child((
        body(title, 15.0, fg),
        if e.discovered && !e.blurb.is_empty() {
            body(e.blurb.clone(), 12.0, col(170, 190, 170))
        } else if !e.discovered {
            body("Not yet discovered".into(), 12.0, col(100, 110, 100))
        } else {
            spacer(0.0)
        },
    ))
}

fn game_over_ui(st: &SharedUi, actions: Arc<Mutex<Vec<UiAction>>>) -> View {
    let a_r = actions.clone();
    let a_t = actions.clone();
    let tr = &st.translations;
    let title = if st.victory {
        t(tr, "you-win", "Ecosystem Thrives!")
    } else {
        t(tr, "you-lose", "Ecosystem Collapsed")
    };
    let status = if !st.end_reason.is_empty() && !st.victory {
        format!("{} — {}", st.status_line, st.end_reason)
    } else {
        st.status_line.clone()
    };

    Column(
        Modifier::new()
            .fill_max_size()
            .justify_content(JustifyContent::CENTER)
            .align_items(AlignItems::CENTER)
            .background(cola(0, 0, 0, 170)),
    )
    .child(panel(
        440.0,
        Column(Modifier::new().align_items(AlignItems::CENTER)).child((
            RText(title).size(30.0).color(RColor::WHITE),
            spacer(8.0),
            body(status, 14.0, col(200, 210, 200)),
            spacer(8.0),
            Row(Modifier::new().gap(10.0)).child((
                chip(
                    format!("{} {}", t(tr, "biodiversity", "Bio"), st.biodiversity),
                    col(40, 80, 50),
                    col(180, 230, 180),
                ),
                chip(
                    format!("{} {}", t(tr, "dew", "Dew"), st.dew),
                    col(40, 70, 100),
                    col(160, 210, 255),
                ),
                chip(
                    format!(
                        "{} {}/{}",
                        t(tr, "discoveries", "Dex"),
                        st.discoveries,
                        st.discoveries_total
                    ),
                    col(90, 80, 40),
                    col(230, 220, 150),
                ),
            )),
            spacer(16.0),
            mk_button(
                &t(tr, "restart-hint", "Restart (R)"),
                col(60, 140, 90),
                move || push(&a_r, UiAction::Restart),
            ),
            mk_button(
                &t(tr, "quit-to-title", "Quit to Title"),
                col(180, 60, 60),
                move || push(&a_t, UiAction::QuitToTitle),
            ),
        )),
    ))
}
