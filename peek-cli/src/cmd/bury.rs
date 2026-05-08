use chrono::Utc;
use peek_core::{
    memorial::{append, Memorial},
    state::{default_memorial_path, default_state_path, PeekState},
    Creature,
};
use rand::Rng;

pub fn run() -> anyhow::Result<()> {
    let path = default_state_path();
    let mut state = PeekState::load(&path).map_err(|e| anyhow::anyhow!("load: {e}"))?;
    let now = Utc::now();
    let Some(c) = state.creature.clone() else {
        println!("peek: no creature to bury.");
        return Ok(());
    };
    let mut tickable = c.clone();
    tickable.tick(now);
    if !tickable.stats.all_zero() {
        println!("peek: {} is still tethered. you can only bury when it has returned to the void.", c.true_name);
        return Ok(());
    }
    let m = Memorial {
        creature_id: c.id,
        true_name: c.true_name.clone(),
        born_at: c.born_at,
        died_at: now,
        final_stage: c.stage,
        chapters_read: c.chapters_read.len() as u32,
    };
    let memorial_path = default_memorial_path();
    append(&memorial_path, m)?;
    let seed: u64 = rand::thread_rng().gen();
    state.creature = Some(Creature::hatch(now, seed));
    state.save(&path).map_err(|e| anyhow::anyhow!("save: {e}"))?;
    let new_name = state.creature.as_ref().map(|c| c.true_name.as_str()).unwrap_or("?");
    println!(
        "peek: buried {}. a fresh egg appears. it calls itself {new_name}.",
        c.true_name,
    );
    Ok(())
}
