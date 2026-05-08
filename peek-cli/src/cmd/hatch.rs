use chrono::Utc;
use peek_core::{
    state::{default_state_path, PeekState},
    Creature,
};
use rand::Rng;

pub fn run() -> anyhow::Result<()> {
    let path = default_state_path();
    let mut state = PeekState::load(&path).map_err(|e| anyhow::anyhow!("load: {e}"))?;
    let seed: u64 = rand::thread_rng().gen();
    state.creature = Some(Creature::hatch(Utc::now(), seed));
    state
        .save(&path)
        .map_err(|e| anyhow::anyhow!("save: {e}"))?;
    let name = state
        .creature
        .as_ref()
        .map(|c| c.true_name.as_str())
        .unwrap_or("?");
    println!("peek: a fresh egg. it calls itself {name}.");
    Ok(())
}
