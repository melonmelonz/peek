use chrono::Utc;
use crossterm::event::{
    self, Event as CtEvent, KeyCode as CtKey, KeyEvent, KeyModifiers as CtMods,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use peek_core::{memorial::append, Creature};
use peek_tui::{
    app::App,
    input::{InputEvent, Key},
    scene::SceneId,
    scene_runner::{build_scene, RunnerOutcome, SceneRunner},
    scenes::DeathScene,
    theme::Theme,
};
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};

pub fn run() -> anyhow::Result<()> {
    let mut app = App::boot()?;

    // First-launch hatch: creature absent.
    let needs_hatch = app.state.creature.is_none();
    if needs_hatch {
        let seed: u64 = rand::thread_rng().gen();
        app.state.creature = Some(Creature::hatch(Utc::now(), seed));
        app.save()?;
    }

    // Lazy decay catch-up since the previous tick.
    let mut died = false;
    if let Some(c) = app.creature_mut() {
        let outcome = c.tick(Utc::now());
        died = outcome.died;
    }

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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, runner, theme);

    disable_raw_mode().ok();
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut runner: SceneRunner,
    theme: Theme,
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(120);
    let mut last_decay = Utc::now();

    loop {
        terminal.draw(|f| {
            let area = f.area();
            runner.render(f, area, app);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            let ev = event::read()?;
            // Allow Ctrl+C anywhere.
            if let CtEvent::Key(k) = &ev {
                if k.modifiers.contains(CtMods::CONTROL) && matches!(k.code, CtKey::Char('c')) {
                    break;
                }
            }
            if let Some(input) = adapt_event(&ev) {
                if let RunnerOutcome::Quit = runner.handle(&input, app) {
                    break;
                }
                drain_memorials(app);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            // Animation tick.
            if let RunnerOutcome::Quit = runner.tick(app) {
                break;
            }
            drain_memorials(app);
            // Decay tick once per minute, regardless of scene.
            let now = Utc::now();
            if (now - last_decay).num_seconds() >= 60 {
                last_decay = now;
                if let Some(c) = app.creature_mut() {
                    let out = c.tick(now);
                    if out.died && runner.scene_id() != SceneId::Death {
                        app.say("death");
                        runner.replace_scene(Box::new(DeathScene::new(theme, app)));
                    }
                }
                let _ = app.save();
            }
        }
    }
    Ok(())
}

fn drain_memorials(app: &mut App) {
    let drained = app.take_pending_memorials();
    for m in drained {
        let _ = append(&app.memorial_path, m);
    }
}

fn adapt_event(ev: &CtEvent) -> Option<InputEvent> {
    let CtEvent::Key(KeyEvent { code, .. }) = ev else {
        return None;
    };
    let key = match code {
        CtKey::Char(c) => Key::Char(*c),
        CtKey::Enter => Key::Enter,
        CtKey::Esc => Key::Esc,
        CtKey::Backspace => Key::Backspace,
        CtKey::Up => Key::Up,
        CtKey::Down => Key::Down,
        CtKey::Left => Key::Left,
        CtKey::Right => Key::Right,
        CtKey::PageUp => Key::PageUp,
        CtKey::PageDown => Key::PageDown,
        _ => Key::Other,
    };
    Some(InputEvent::Key(key))
}
