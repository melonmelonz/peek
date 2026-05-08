use chrono::Utc;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use peek_core::Creature;
use peek_tui::{
    app::App,
    scene::{Scene, SceneAction, SceneId},
    scenes::{DeathScene, HatchScene, IdleScene, QuizScene, ReadScene},
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
    let initial: Box<dyn Scene> = if needs_hatch {
        Box::new(HatchScene::new(theme))
    } else if died {
        Box::new(DeathScene::new(theme, &app))
    } else {
        Box::new(IdleScene::new(theme))
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, initial, theme);

    disable_raw_mode().ok();
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut scene: Box<dyn Scene>,
    theme: Theme,
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(120);
    let mut last_decay = Utc::now();

    loop {
        terminal.draw(|f| {
            let area = f.size();
            scene.render(f, area, app);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            let ev = event::read()?;
            // Allow Ctrl+C anywhere.
            if let Event::Key(k) = &ev {
                if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    && matches!(k.code, crossterm::event::KeyCode::Char('c'))
                {
                    break;
                }
            }
            match scene.handle(&ev, app) {
                SceneAction::Stay => {}
                SceneAction::Quit => break,
                SceneAction::Goto(id) => {
                    scene = build_scene(id, theme, app);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            // Animation tick.
            match scene.tick(app) {
                SceneAction::Stay => {}
                SceneAction::Quit => break,
                SceneAction::Goto(id) => {
                    scene = build_scene(id, theme, app);
                }
            }
            // Decay tick once per minute, regardless of scene.
            let now = Utc::now();
            if (now - last_decay).num_seconds() >= 60 {
                last_decay = now;
                if let Some(c) = app.creature_mut() {
                    let out = c.tick(now);
                    if out.died && scene.id() != SceneId::Death {
                        app.say("death");
                        scene = Box::new(DeathScene::new(theme, app));
                    }
                }
                let _ = app.save();
            }
        }
    }
    Ok(())
}

fn build_scene(id: SceneId, theme: Theme, app: &mut App) -> Box<dyn Scene> {
    match id {
        SceneId::Idle => Box::new(IdleScene::new(theme)),
        SceneId::Hatch => Box::new(HatchScene::new(theme)),
        SceneId::Quiz => Box::new(QuizScene::new(theme, peek_tui::scenes::quiz::QuizMode::Feed, app)),
        SceneId::Read => Box::new(ReadScene::new(theme, app)),
        SceneId::Death => Box::new(DeathScene::new(theme, app)),
    }
}
