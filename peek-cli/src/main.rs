//! PEEK static-musl CLI.
//!
//! `peek` with no subcommand opens the TUI. Other subcommands are designed
//! for cron / scripting use.

use clap::{Parser, Subcommand};

mod cmd;

#[derive(Parser, Debug)]
#[command(
    name = "peek",
    version,
    about = "PEEK Examines Embedded Kernels: an eldritch tamagotchi for offline systems-curriculum study.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Open the TUI (default).
    Tui,
    /// Apply lazy decay since the last tick. Cron-friendly.
    Tick,
    /// Force a fresh egg (destroys the current creature without burying).
    Hatch,
    /// Write a memorial and roll a new egg. Only valid when the creature is dead.
    Bury,
    /// Print the state file path and a sample crontab line.
    Path,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd.unwrap_or(Cmd::Tui) {
        Cmd::Tui => cmd::tui::run(),
        Cmd::Tick => cmd::tick::run(),
        Cmd::Hatch => cmd::hatch::run(),
        Cmd::Bury => cmd::bury::run(),
        Cmd::Path => cmd::path::run(),
    }
}
