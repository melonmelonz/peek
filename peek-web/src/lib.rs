//! PEEK browser build.
//!
//! This crate compiles to a wasm32 cdylib, loads/saves state in
//! `localStorage`, and drives the same `peek-tui` scenes the native CLI
//! uses. Trunk is the build pipeline; see `Trunk.toml`.

use std::cell::RefCell;
use std::rc::Rc;

use chrono::Utc;
use peek_core::Creature;
use peek_tui::{
    app::App,
    input::{InputEvent, Key},
    scene::SceneId,
    scene_runner::{build_scene, RunnerOutcome, SceneRunner},
    scenes::{DeathScene, EvolveScene},
    theme::Theme,
};
use rand::Rng;
use ratzilla::event::{KeyCode as RzKey, KeyEvent as RzKeyEvent};
use ratzilla::ratatui::Terminal;
use ratzilla::{DomBackend, WebRenderer};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

mod storage;

use storage::{load_memorials, load_state, save_memorials, save_state};

/// Logical tick interval in milliseconds (matches the native build's 120ms).
const TICK_MS: f64 = 120.0;
/// Decay tick interval in milliseconds (60s, matches native).
const DECAY_MS: f64 = 60_000.0;

/// Browser entry point. Trunk wires this up via `data-trunk` on a script tag,
/// or you can call `init()` from JS after loading the wasm bundle.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    boot().map_err(|e| JsValue::from_str(&format!("peek-web boot: {e}")))
}

fn boot() -> anyhow::Result<()> {
    // Load state from localStorage. A missing or corrupt entry produces a
    // fresh state — the same behavior the native build has when the state
    // file is absent.
    let mut state = load_state();

    // First-launch hatch: creature absent.
    let needs_hatch = state.creature.is_none();
    if needs_hatch {
        let seed: u64 = rand::thread_rng().gen();
        state.creature = Some(Creature::hatch(Utc::now(), seed));
    }

    // Lazy decay catch-up since the previous tick.
    let mut died = false;
    if let Some(c) = state.creature.as_mut() {
        let outcome = c.tick(Utc::now());
        died = outcome.died;
    }

    let mut app = App::new(state, PathBuf::new(), PathBuf::new())?;
    save_state(&app.state);

    let theme = Theme::neon();
    let initial_id = if needs_hatch {
        SceneId::Intro
    } else if died {
        SceneId::Death
    } else {
        SceneId::Idle
    };
    let initial = build_scene(initial_id, theme, &mut app);
    let runner = SceneRunner::new(initial, theme);

    let backend =
        DomBackend::new().map_err(|e| anyhow::anyhow!("DomBackend init failed: {e:?}"))?;
    let terminal =
        Terminal::new(backend).map_err(|e| anyhow::anyhow!("Terminal init failed: {e:?}"))?;

    let shared = Rc::new(RefCell::new(WebState {
        app,
        runner,
        theme,
        last_tick_ms: now_ms(),
        last_decay_ms: now_ms(),
    }));

    // Key handler: convert ratzilla KeyEvent → InputEvent → runner.handle.
    let key_state = shared.clone();
    terminal.on_key_event(move |ev: RzKeyEvent| {
        let mut guard = match key_state.try_borrow_mut() {
            Ok(s) => s,
            Err(_) => return,
        };
        let s: &mut WebState = &mut guard;
        if let Some(input) = adapt_key(&ev) {
            match s.runner.handle(&input, &mut s.app) {
                RunnerOutcome::Quit => {
                    // Browsers can't really "quit" a tab; treat as soft reset.
                    s.app.current_dialogue = Some("press a key to keep going.".into());
                }
                RunnerOutcome::Continue => {}
            }
            persist_after(&mut s.app);
        }
    });

    // Render loop: drive ticks based on elapsed wall-clock time, then
    // render the current scene. ratzilla calls this every animation frame.
    let render_state = shared.clone();
    terminal.draw_web(move |frame| {
        let mut guard = match render_state.try_borrow_mut() {
            Ok(s) => s,
            Err(_) => return,
        };
        let s: &mut WebState = &mut guard;
        let now = now_ms();

        // Logical tick.
        if now - s.last_tick_ms >= TICK_MS {
            s.last_tick_ms = now;
            let _ = s.runner.tick(&mut s.app);
            persist_after(&mut s.app);
        }

        // Decay tick once per minute, regardless of scene.
        if now - s.last_decay_ms >= DECAY_MS {
            s.last_decay_ms = now;
            let now_dt = Utc::now();
            let mut just_died = false;
            let mut advanced_to = None;
            let mut from_stage = None;
            if let Some(c) = s.app.creature_mut() {
                let prior = c.stage;
                let out = c.tick(now_dt);
                just_died = out.died;
                if out.advanced {
                    advanced_to = Some(c.stage);
                    from_stage = Some(prior);
                }
            }
            if just_died && s.runner.scene_id() != SceneId::Death {
                s.app.say("death");
                let theme = s.theme;
                let scene: Box<dyn peek_tui::scene::Scene> =
                    Box::new(DeathScene::new(theme, &s.app));
                s.runner.replace_scene(scene);
            } else if let (Some(to), Some(from)) = (advanced_to, from_stage) {
                if s.runner.scene_id() == SceneId::Idle {
                    s.app.say("stage_up");
                    let theme = s.theme;
                    let scene: Box<dyn peek_tui::scene::Scene> =
                        Box::new(EvolveScene::new(theme, from, to));
                    s.runner.replace_scene(scene);
                }
            }
            save_state(&s.app.state);
        }

        let area = frame.area();
        s.runner.render(frame, area, &s.app);
    });

    Ok(())
}

struct WebState {
    app: App,
    runner: SceneRunner,
    theme: Theme,
    last_tick_ms: f64,
    last_decay_ms: f64,
}

/// Drain memorials produced by the latest event/tick and persist whatever
/// state changed back to localStorage.
fn persist_after(app: &mut App) {
    let mut memorials = load_memorials();
    let drained = app.take_pending_memorials();
    if !drained.is_empty() {
        memorials.extend(drained);
        save_memorials(&memorials);
    }
    save_state(&app.state);
}

fn adapt_key(ev: &RzKeyEvent) -> Option<InputEvent> {
    let key = match ev.code {
        RzKey::Char(c) => Key::Char(c),
        RzKey::Enter => Key::Enter,
        RzKey::Esc => Key::Esc,
        RzKey::Backspace => Key::Backspace,
        RzKey::Up => Key::Up,
        RzKey::Down => Key::Down,
        RzKey::Left => Key::Left,
        RzKey::Right => Key::Right,
        RzKey::PageUp => Key::PageUp,
        RzKey::PageDown => Key::PageDown,
        _ => Key::Other,
    };
    Some(InputEvent::Key(key))
}

fn now_ms() -> f64 {
    js_sys::Date::now()
}

/// Reset state — exposed to JS so the embed page can offer a "restart"
/// button without making the user clear their browser storage.
#[wasm_bindgen]
pub fn peek_reset() {
    storage::clear_all();
}
