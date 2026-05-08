use chrono::Utc;
use peek_core::{
    state::{default_state_path, PeekState},
    Stage,
};

pub fn run() -> anyhow::Result<()> {
    let path = default_state_path();
    let mut state = PeekState::load(&path).map_err(|e| anyhow::anyhow!("load: {e}"))?;
    let Some(creature) = state.creature.as_mut() else {
        println!("peek: no creature yet. run `peek` once to hatch.");
        return Ok(());
    };
    let outcome = creature.tick(Utc::now());
    let stage_name = creature.stage.name();
    state
        .save(&path)
        .map_err(|e| anyhow::anyhow!("save: {e}"))?;
    let died = outcome.died;
    let starting_threshold = matches!(state.creature.as_ref().map(|c| c.stage), Some(Stage::Egg));
    let _ = starting_threshold;
    println!("peek: tick applied. stage={stage_name} died={died}",);
    Ok(())
}
