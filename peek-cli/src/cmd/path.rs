use peek_core::state::{default_memorial_path, default_state_path};

pub fn run() -> anyhow::Result<()> {
    let state = default_state_path();
    let memorial = default_memorial_path();
    println!("state file:    {}", state.display());
    println!("memorial file: {}", memorial.display());
    println!();
    println!("# sample crontab line (every 5 minutes):");
    println!("*/5 * * * * /usr/local/bin/peek tick >/dev/null 2>&1");
    Ok(())
}
