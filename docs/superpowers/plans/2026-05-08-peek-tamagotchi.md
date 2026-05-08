# PEEK tamagotchi v1 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a static-musl Rust TUI tamagotchi (engagement layer for PEEK Examines Embedded Kernels) with three-stat care loop, hybrid question system, SM-2 lite recall, and GitHub release distribution linked from goolz.org/next-chapter.

**Architecture:** Cargo workspace with four crates: `peek-core` (pure logic), `peek-content` (embedded curriculum + sprites + bank), `peek-tui` (ratatui scenes), `peek-cli` (musl-static binary). Lazy-tick decay, no daemon, no network.

**Tech Stack:** Rust 1.78+, ratatui, crossterm, rust-embed, ron, serde, chrono, uuid, rand_chacha, clap, insta, assert_cmd.

Spec lives at `docs/superpowers/specs/2026-05-08-peek-tamagotchi-design.md`.

---

## Phase A: Workspace foundation

### Task A1: Create workspace + four-crate skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `peek-core/Cargo.toml`
- Create: `peek-core/src/lib.rs`
- Create: `peek-content/Cargo.toml`
- Create: `peek-content/src/lib.rs`
- Create: `peek-tui/Cargo.toml`
- Create: `peek-tui/src/lib.rs`
- Create: `peek-cli/Cargo.toml`
- Create: `peek-cli/src/main.rs`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`

- [ ] **Step 1: Workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["peek-core", "peek-content", "peek-tui", "peek-cli"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
authors = ["Penn Porterfield"]
repository = "https://github.com/melonmelonz/peek"
description = "PEEK Examines Embedded Kernels: an eldritch tamagotchi TUI for offline systems-curriculum study."

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
ron = "0.8"
chrono = { version = "0.4", default-features = false, features = ["clock", "serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
rand = "0.8"
rand_chacha = "0.3"
ratatui = "0.27"
crossterm = "0.27"
rust-embed = "8"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
thiserror = "1"
pulldown-cmark = "0.10"
insta = "1"
assert_cmd = "2"
proptest = "1"

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

- [ ] **Step 2: `rust-toolchain.toml`**

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
```

- [ ] **Step 3: `.gitignore`**

```
/target
**/*.rs.bk
*.pdb
.DS_Store
.vscode/
.idea/
*.swp
.superpowers/
```

- [ ] **Step 4: Each crate `Cargo.toml`** (substitute name)

`peek-core/Cargo.toml`:
```toml
[package]
name = "peek-core"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "PEEK core domain logic: creature, decay, recall scheduling."

[dependencies]
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
rand = { workspace = true }
rand_chacha = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
ron = { workspace = true }
proptest = { workspace = true }
```

`peek-content/Cargo.toml`:
```toml
[package]
name = "peek-content"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "PEEK embedded curriculum, question bank, and sprite art."

[dependencies]
peek-core = { path = "../peek-core" }
serde = { workspace = true }
ron = { workspace = true }
rust-embed = { workspace = true }
pulldown-cmark = { workspace = true }
anyhow = { workspace = true }
```

`peek-tui/Cargo.toml`:
```toml
[package]
name = "peek-tui"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "PEEK TUI scenes and widgets (ratatui)."

[dependencies]
peek-core = { path = "../peek-core" }
peek-content = { path = "../peek-content" }
ratatui = { workspace = true }
crossterm = { workspace = true }
chrono = { workspace = true }
rand = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
insta = { workspace = true, features = ["yaml"] }
```

`peek-cli/Cargo.toml`:
```toml
[package]
name = "peek-cli"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "PEEK static-musl CLI."

[[bin]]
name = "peek"
path = "src/main.rs"

[dependencies]
peek-core = { path = "../peek-core" }
peek-content = { path = "../peek-content" }
peek-tui = { path = "../peek-tui" }
clap = { workspace = true }
anyhow = { workspace = true }
chrono = { workspace = true }
ron = { workspace = true }
crossterm = { workspace = true }
ratatui = { workspace = true }

[dev-dependencies]
assert_cmd = { workspace = true }
```

- [ ] **Step 5: Each `lib.rs` / `main.rs`** stubs

```rust
// peek-core/src/lib.rs
//! PEEK core domain logic.
```
```rust
// peek-content/src/lib.rs
//! PEEK embedded content.
```
```rust
// peek-tui/src/lib.rs
//! PEEK TUI scenes and widgets.
```
```rust
// peek-cli/src/main.rs
fn main() -> anyhow::Result<()> {
    println!("peek v0.1.0 (scaffold)");
    Ok(())
}
```

- [ ] **Step 6: Verify build**

Run: `cargo build`
Expected: workspace compiles cleanly.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml peek-core peek-content peek-tui peek-cli rust-toolchain.toml .gitignore
git commit -m "chore: scaffold workspace and four-crate layout"
```

### Task A2: Licenses, README skeleton, vocabulary lint

**Files:**
- Create: `LICENSE-MIT`, `LICENSE-APACHE`, `README.md`, `scripts/vocab-lint.sh`

- [ ] **Step 1:** Drop standard MIT and Apache-2.0 license texts into the two files (use `https://opensource.org/license/mit/` and `https://opensource.org/license/apache-2-0/` boilerplate, with copyright "Copyright (c) 2026 Penn Porterfield").

- [ ] **Step 2:** `README.md`:

```markdown
# PEEK

> **PEEK Examines Embedded Kernels.** A neon-90s TUI tamagotchi that lives in a zellij pane. Raise a tiny eldritch creature by walking through a systems-curriculum: read chapters with it, answer questions correctly, visit often enough that it stays tethered to this plane.

This is the engagement layer for PEEK proper, the static-musl systems-curriculum companion in the rust-spi-tinydoom capstone (Slack Next Chapter, May-Aug 2026). v1 ships the tamagotchi; the book reader, LaTeX/PDF render, and static-site renderer follow.

## Install

Prebuilt static-musl binaries on the [releases page](https://github.com/melonmelonz/peek/releases). x86_64 and aarch64.

```sh
curl -L -o peek https://github.com/melonmelonz/peek/releases/latest/download/peek-x86_64-unknown-linux-musl
chmod +x peek
./peek
```

## Build from source

```sh
cargo build --release
./target/release/peek
```

## Run

```sh
peek           # open the TUI
peek tick      # advance decay (cron-friendly, no UI)
peek path      # show state file path and a sample crontab line
peek hatch     # force a fresh egg (destroys current creature)
peek bury      # write memorial and roll new egg (only if creature is dead)
```

## License

Code: MIT OR Apache-2.0.
Curriculum text under `peek-content/chapters/`: CC BY 4.0.
```

- [ ] **Step 3:** `scripts/vocab-lint.sh`:

```bash
#!/usr/bin/env bash
# Fails CI if blocked vocabulary appears in lore/UI strings.
set -euo pipefail
BLOCKED='\b(exploit|crack|breach|hack|0day|payload|backdoor)\b'
if grep -RIn -E "$BLOCKED" --include='*.rs' --include='*.ron' --include='*.md' \
    peek-core peek-content peek-tui peek-cli 2>/dev/null; then
  echo "vocab-lint: blocked words found above. PEEK is engineering, not hacking." >&2
  exit 1
fi
echo "vocab-lint: clean."
```
Make it executable: `chmod +x scripts/vocab-lint.sh`.

- [ ] **Step 4: Run lint locally:** `bash scripts/vocab-lint.sh`. Expected: `vocab-lint: clean.`

- [ ] **Step 5: Commit:** `git add LICENSE-MIT LICENSE-APACHE README.md scripts/ && git commit -m "docs: license, README skeleton, vocab lint script"`.

### Task A3: GitHub Actions CI

**Files:** `.github/workflows/ci.yml`, `.github/workflows/release.yml`

- [ ] **Step 1: `.github/workflows/ci.yml`:**

```yaml
name: ci
on:
  push: { branches: [main] }
  pull_request:
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
      - run: bash scripts/vocab-lint.sh
```

- [ ] **Step 2: `.github/workflows/release.yml`:**

```yaml
name: release
on:
  push:
    tags: ["v*"]
jobs:
  build:
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-musl
          - aarch64-unknown-linux-musl
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - uses: Swatinem/rust-cache@v2
      - name: Install musl tools
        run: sudo apt-get update && sudo apt-get install -y musl-tools gcc-aarch64-linux-gnu
      - name: Cargo config for cross
        run: |
          mkdir -p .cargo
          cat <<EOF >> .cargo/config.toml
          [target.aarch64-unknown-linux-musl]
          linker = "aarch64-linux-gnu-gcc"
          EOF
      - run: cargo build --release --target ${{ matrix.target }} -p peek-cli
      - name: Rename
        run: |
          mkdir -p dist
          cp target/${{ matrix.target }}/release/peek dist/peek-${{ matrix.target }}
          (cd dist && sha256sum peek-${{ matrix.target }} > peek-${{ matrix.target }}.sha256)
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            dist/peek-${{ matrix.target }}
            dist/peek-${{ matrix.target }}.sha256
```

- [ ] **Step 3: Commit:** `git add .github/ && git commit -m "ci: workspace CI and tagged-release musl matrix"`.

---

## Phase B: peek-core domain (TDD)

### Task B1: Stats + clamping

**Files:** `peek-core/src/stats.rs`, `peek-core/src/lib.rs`

- [ ] **Step 1: Failing test** at `peek-core/src/stats.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stats { pub nourishment: f32, pub tether: f32, pub lucidity: f32 }

impl Stats {
    pub fn new_full() -> Self { Self { nourishment: 1.0, tether: 1.0, lucidity: 1.0 } }
    pub fn clamp(&mut self) {
        for v in [&mut self.nourishment, &mut self.tether, &mut self.lucidity] {
            *v = v.clamp(0.0, 1.0);
        }
    }
    pub fn any_zero(&self) -> bool {
        self.nourishment <= 0.0 || self.tether <= 0.0 || self.lucidity <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn clamp_to_unit() {
        let mut s = Stats { nourishment: 1.5, tether: -0.2, lucidity: 0.5 };
        s.clamp();
        assert_eq!(s, Stats { nourishment: 1.0, tether: 0.0, lucidity: 0.5 });
    }
    #[test] fn any_zero_detected() {
        assert!(Stats { nourishment: 0.0, tether: 1.0, lucidity: 1.0 }.any_zero());
        assert!(!Stats::new_full().any_zero());
    }
}
```

- [ ] **Step 2:** Add `pub mod stats;` and `pub use stats::Stats;` to `peek-core/src/lib.rs`.

- [ ] **Step 3:** Run `cargo test -p peek-core stats`. Expected: 2 passes.

- [ ] **Step 4: Commit:** `git add peek-core/src && git commit -m "feat(core): Stats with clamp and any_zero"`.

### Task B2: Decay model

**Files:** `peek-core/src/decay.rs`, `peek-core/src/lib.rs`

- [ ] **Step 1:** `peek-core/src/decay.rs`:

```rust
use crate::Stats;

#[derive(Debug, Clone, Copy)]
pub struct DecayRates {
    /// Half-life in hours for each stat.
    pub nourishment_h: f32,
    pub tether_h: f32,
    pub lucidity_h: f32,
}

impl DecayRates {
    pub const DEFAULT: Self = Self { nourishment_h: 36.0, tether_h: 72.0, lucidity_h: 60.0 };
}

/// Apply exponential decay to `stats` over `elapsed_seconds`.
pub fn apply_decay(stats: &mut Stats, rates: DecayRates, elapsed_seconds: f64) {
    let h = (elapsed_seconds / 3600.0) as f32;
    let mul = |half: f32| (-h * std::f32::consts::LN_2 / half).exp();
    stats.nourishment *= mul(rates.nourishment_h);
    stats.tether       *= mul(rates.tether_h);
    stats.lucidity     *= mul(rates.lucidity_h);
    stats.clamp();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn half_life_halves() {
        let mut s = Stats::new_full();
        let r = DecayRates::DEFAULT;
        apply_decay(&mut s, r, (r.nourishment_h as f64) * 3600.0);
        assert!((s.nourishment - 0.5).abs() < 0.001);
    }
    #[test] fn no_negative() {
        let mut s = Stats { nourishment: 0.01, tether: 0.01, lucidity: 0.01 };
        apply_decay(&mut s, DecayRates::DEFAULT, 1_000_000.0);
        assert!(s.nourishment >= 0.0 && s.tether >= 0.0 && s.lucidity >= 0.0);
    }
}
```

- [ ] **Step 2:** Add `pub mod decay;` to `lib.rs`.

- [ ] **Step 3:** `cargo test -p peek-core decay`. Expected: pass.

- [ ] **Step 4: Commit:** `git commit -am "feat(core): exponential decay with per-stat half-lives"`.

### Task B3: Stage and Mood enums

**Files:** `peek-core/src/stage.rs`, `peek-core/src/mood.rs`

- [ ] **Step 1:** `stage.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage { Egg, Sprout, Knot, Mawling, Conduit, Cogent }

impl Stage {
    pub const ORDER: [Stage; 6] = [Stage::Egg, Stage::Sprout, Stage::Knot, Stage::Mawling, Stage::Conduit, Stage::Cogent];
    pub fn index(self) -> u8 { Self::ORDER.iter().position(|s| *s == self).unwrap() as u8 }
    pub fn next(self) -> Option<Stage> {
        let i = self.index() as usize + 1;
        Self::ORDER.get(i).copied()
    }
    pub fn name(self) -> &'static str {
        match self { Stage::Egg=>"egg", Stage::Sprout=>"sprout", Stage::Knot=>"knot",
            Stage::Mawling=>"mawling", Stage::Conduit=>"conduit", Stage::Cogent=>"cogent" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn next_chain_terminates() {
        let mut s = Stage::Egg;
        let mut hops = 0;
        while let Some(n) = s.next() { s = n; hops += 1; }
        assert_eq!(hops, 5);
        assert_eq!(s, Stage::Cogent);
    }
}
```

- [ ] **Step 2:** `mood.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::Stats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood { Anxious, Lucid, Drifting, Ravenous, Reverent }

impl Mood {
    /// Derive mood from current stats. `recently_advanced` flags the post-stage-up window.
    pub fn from_stats(stats: &Stats, recently_advanced: bool) -> Mood {
        if recently_advanced { return Mood::Reverent; }
        if stats.nourishment < 0.2 { return Mood::Ravenous; }
        if stats.tether       < 0.2 { return Mood::Drifting; }
        if stats.lucidity     > 0.8 { return Mood::Lucid; }
        Mood::Anxious
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ravenous_when_hungry() {
        let s = Stats { nourishment: 0.1, tether: 1.0, lucidity: 0.5 };
        assert_eq!(Mood::from_stats(&s, false), Mood::Ravenous);
    }
    #[test] fn reverent_after_advance() {
        let s = Stats::new_full();
        assert_eq!(Mood::from_stats(&s, true), Mood::Reverent);
    }
}
```

- [ ] **Step 3:** Add `pub mod stage; pub mod mood;` plus re-exports.

- [ ] **Step 4:** `cargo test -p peek-core`. All pass.

- [ ] **Step 5: Commit:** `git commit -am "feat(core): Stage and Mood with derivation rules"`.

### Task B4: Mutation enum + sprite overlay map

**Files:** `peek-core/src/mutation.rs`

- [ ] **Step 1:** Implement `Mutation` enum with variants `ExtraEye, ThirdMouth, Tendril { count: u8 }, InvertedSpiral, NoneAtAll, Crown`. Add `pub fn label(&self) -> &'static str`. Random rolling will be added in Task B6 (creature module).

- [ ] **Step 2:** Test:

```rust
#[test] fn labels_exist() {
    assert_eq!(Mutation::ExtraEye.label(), "extra eye");
    assert_eq!(Mutation::Tendril { count: 3 }.label(), "tendril");
}
```

- [ ] **Step 3:** Commit: `feat(core): Mutation enum with labels`.

### Task B5: Creature with seed-derived hatch

**Files:** `peek-core/src/creature.rs`

- [ ] **Step 1:** Define `Creature` struct mirroring spec section 5.1, plus `Creature::hatch(now: DateTime<Utc>, seed: u64) -> Creature` that:
  - Derives a ChaCha20 RNG from seed.
  - Generates `true_name` from a syllable pool (eldritch consonants + vowels), 2 syllables + apostrophe + 1 syllable.
  - Sets stage to Egg (the `Sprout` transition happens after the first user interaction).
  - `mood = Anxious`, `stats = Stats::new_full()`, `mutations = vec![]`.
  - `id = Uuid::new_v4()` (separate from seed; seed only governs derived traits).

- [ ] **Step 2:** Property test: same seed -> same `true_name` and same first-stage-up mutation roll.

```rust
#[test] fn deterministic_name() {
    let t = chrono::Utc::now();
    let a = Creature::hatch(t, 42).true_name;
    let b = Creature::hatch(t, 42).true_name;
    assert_eq!(a, b);
}
```

- [ ] **Step 3:** Add `Creature::advance_stage(&mut self, rng: &mut impl rand::RngCore)` that bumps stage and rolls 0-2 new mutations from the rng.

- [ ] **Step 4:** Commit: `feat(core): Creature with deterministic hatch`.

### Task B6: PeekState save/load (RON)

**Files:** `peek-core/src/state.rs`

- [ ] **Step 1:** Define `PeekState { schema_version: u32, creature: Option<Creature>, recall: Vec<RecallRecord>, recent_dialogue: VecDeque<DialogueEvent>, install_id: Uuid }`. Stub `RecallRecord` and `DialogueEvent` minimal so file compiles (full impls in B7/B8).

- [ ] **Step 2:** Implement `PeekState::load(path: &Path) -> Result<Self, StateError>` and `save(&self, path)`. Use atomic write (tmp file + rename). Use `ron::ser::PrettyConfig`.

- [ ] **Step 3:** Test: round-trip a state with one creature through `save` then `load`. Compare creature ids.

- [ ] **Step 4:** Test: `path` helper that returns `$XDG_STATE_HOME/peek/state.ron` (fallback `$HOME/.local/state/peek/state.ron`).

- [ ] **Step 5:** Commit: `feat(core): PeekState atomic RON save/load`.

### Task B7: Question types

**Files:** `peek-core/src/question.rs`

- [ ] **Step 1:** Define `ChapterId(pub Cow<'static, str>)`, `QuestionId(pub String)`, `Difficulty(pub u8)`, `Question`, `QuestionKind` per spec section 5.3. Derive `Serialize, Deserialize, Clone, Debug`.

- [ ] **Step 2:** Implement `Question::evaluate(&self, answer: &str) -> AttemptResult` where `AttemptResult { correct: bool, reveal: String }`. For each kind:
  - `MultipleChoice`: accept "a"/"b"/"c"... case-insensitive, map to index, compare to `correct`.
  - `FillBlank`: trim, lowercase, accept any of `accept`.
  - `ShortNumeric`: parse f64, check in `accept_range`.
  - `TraceProgram`: trim trailing whitespace, compare exact string.

- [ ] **Step 3:** Tests for each kind, both correct and incorrect.

- [ ] **Step 4:** Commit: `feat(core): Question kinds with evaluation`.

### Task B8: Recall (SM-2 lite)

**Files:** `peek-core/src/recall.rs`

- [ ] **Step 1:** Define `RecallRecord` per spec 5.4. Implement `RecallRecord::new_for(question: QuestionId, now)` and `update(&mut self, correct: bool, now)` with SM-2 lite:
  - Correct: `interval_hours *= ease`; `streak += 1`. If `streak == 1`, `interval_hours = 6.0`.
  - Wrong: `interval_hours = 1.0`; `streak = 0`; `ease = (ease - 0.2).max(1.3)`.
  - `next_due = last_seen + interval`.

- [ ] **Step 2:** `pub fn due_now(records: &[RecallRecord], now) -> Vec<&RecallRecord>` ordered by `next_due` ascending.

- [ ] **Step 3:** Property test: ease never falls below 1.3.

- [ ] **Step 4:** Commit: `feat(core): SM-2 lite recall scheduling`.

### Task B9: Care actions and effects

**Files:** `peek-core/src/care.rs`

- [ ] **Step 1:** Define `CareAction { Feed { result: AttemptResult, was_new: bool }, Tend, Read { chapter_seen_before: bool }, Quiz { result: AttemptResult } }`. Implement `apply(creature: &mut Creature, action: CareAction)` that updates stats per spec section 6:
  - Feed correct: `+0.25` nourishment. `was_new` true: `+0.05` lucidity.
  - Feed wrong: `-0.05` tether.
  - Tend: `+0.10` tether.
  - Read new: `+0.20` lucidity.
  - Read repeat: `+0.05` tether.
  - Quiz correct (new): `+0.20` nourishment + `+0.10` lucidity.
  - Quiz wrong: `-0.05` tether.
  - Always `creature.stats.clamp()`.

- [ ] **Step 2:** Tests covering each branch.

- [ ] **Step 3:** Implement `Creature::tick(&mut self, now)` that applies decay since `last_tick`, then checks if all-zero condition met for >24h (record this on `Creature::zero_floor_since: Option<DateTime<Utc>>`). Returns `TickOutcome { died: bool, advanced: bool }`.

- [ ] **Step 4:** Commit: `feat(core): care actions, decay tick, death detection`.

### Task B10: Memorial

**Files:** `peek-core/src/memorial.rs`

- [ ] **Step 1:** `pub struct Memorial { creature_id: Uuid, true_name: String, born_at, died_at, final_stage: Stage, chapters_read: u32 }`. Append-only RON list at `memorials.ron` next to state.

- [ ] **Step 2:** `pub fn append(path: &Path, m: Memorial) -> Result<()>` plus `pub fn load_all(path) -> Vec<Memorial>` (errors empty).

- [ ] **Step 3:** Round-trip test.

- [ ] **Step 4:** Commit: `feat(core): memorial append-only log`.

---

## Phase C: peek-content

### Task C1: rust-embed for chapters and bank

**Files:** `peek-content/src/lib.rs`, `peek-content/chapters/ch01-programs-and-os.md`, `peek-content/questions/bank.ron`, `peek-content/sprites/`, `peek-content/dialogue.ron`

- [ ] **Step 1:** `peek-content/src/lib.rs`:

```rust
use rust_embed::RustEmbed;
use peek_core::{Question, ChapterId};

#[derive(RustEmbed)]
#[folder = "chapters/"]
struct Chapters;

#[derive(RustEmbed)]
#[folder = "questions/"]
struct Questions;

#[derive(RustEmbed)]
#[folder = "sprites/"]
struct Sprites;

#[derive(RustEmbed)]
#[folder = "dialogue/"]
struct Dialogue;

pub struct Curriculum;
impl Curriculum {
    pub fn chapter_ids() -> Vec<ChapterId> {
        Chapters::iter()
            .filter(|p| p.ends_with(".md"))
            .map(|p| ChapterId::from(p.trim_end_matches(".md").to_string()))
            .collect()
    }
    pub fn chapter_text(id: &ChapterId) -> Option<String> {
        let path = format!("{}.md", id.as_str());
        Chapters::get(&path).map(|f| String::from_utf8_lossy(&f.data).to_string())
    }
}

pub struct QuestionBank;
impl QuestionBank {
    pub fn load() -> anyhow::Result<Vec<Question>> {
        let raw = Questions::get("bank.ron").ok_or_else(|| anyhow::anyhow!("bank.ron missing"))?;
        Ok(ron::from_str(std::str::from_utf8(&raw.data)?)?)
    }
}

pub struct SpriteSet;
impl SpriteSet {
    pub fn frame(stage: peek_core::Stage, mood: peek_core::Mood, frame_idx: u8) -> Option<String> {
        let path = format!("{}_{}_{}.txt", stage.name(), mood_label(mood), frame_idx);
        Sprites::get(&path).map(|f| String::from_utf8_lossy(&f.data).to_string())
    }
}
fn mood_label(m: peek_core::Mood) -> &'static str {
    match m { peek_core::Mood::Anxious=>"anxious", peek_core::Mood::Lucid=>"lucid",
        peek_core::Mood::Drifting=>"drifting", peek_core::Mood::Ravenous=>"ravenous",
        peek_core::Mood::Reverent=>"reverent" }
}

pub struct DialogueLines;
impl DialogueLines {
    pub fn load() -> anyhow::Result<Vec<DialogueLine>> {
        let raw = Dialogue::get("lines.ron").ok_or_else(|| anyhow::anyhow!("dialogue/lines.ron missing"))?;
        Ok(ron::from_str(std::str::from_utf8(&raw.data)?)?)
    }
}

#[derive(serde::Deserialize)]
pub struct DialogueLine { pub event: String, pub stage: Option<String>, pub text: String, #[serde(default)] pub memorial_aware: bool }
```

Add `peek-content` re-exports to allow callers `use peek_content::{Curriculum, QuestionBank, SpriteSet, DialogueLines};`.

(Note: requires `ChapterId::as_str()` and `From<String>` impls on `peek-core` ChapterId. Add those to Task B7's wrap-up if not already present.)

- [ ] **Step 2:** Test: `Curriculum::chapter_ids()` returns at least one chapter after C2 ships content.

- [ ] **Step 3:** Commit: `feat(content): rust-embed loaders for chapters, bank, sprites, dialogue`.

### Task C2: First chapter markdown

**Files:** `peek-content/chapters/ch01-programs-and-os.md`, `peek-content/chapters/ch02-syscalls.md`, `peek-content/chapters/ch03-memory-and-mmap.md`

- [ ] **Step 1:** Author each chapter as ~500-800 words of plain markdown, matching the PEEK roadmap topics (programs vs OS, syscalls, mmap and friends). No exploitation vocabulary; engineering register. Each chapter ends with a `## key terms` list (plain bullet list of terms a question generator can sample).

- [ ] **Step 2:** Commit: `content: first three curriculum chapters`.

### Task C3: Question bank seed

**Files:** `peek-content/questions/bank.ron`

- [ ] **Step 1:** Author 18 questions: 6 per chapter, 2 each at difficulty 1, 3, 5. Mix of `MultipleChoice`, `FillBlank`, `ShortNumeric`. Each has an `explanation` that pulls from the chapter prose. Example shape:

```ron
[
  (
    id: "ch01-q01",
    chapter: "ch01-programs-and-os",
    difficulty: 1,
    kind: MultipleChoice(
      prompt: "What does an operating system do that a program by itself does not?",
      options: ["arithmetic", "share hardware between programs", "encrypt files", "render graphics"],
      correct: 1,
    ),
    explanation: "An OS multiplexes hardware (CPU, memory, devices) across multiple programs.",
    tags: ["intro"],
  ),
  // ... 17 more
]
```

- [ ] **Step 2:** Test in `peek-content/tests/bank_loads.rs`: `QuestionBank::load().unwrap().len() >= 18`. Each `Question::evaluate` on the documented-correct answer returns `correct: true`.

- [ ] **Step 3:** Commit: `content: 18-question seed bank across three chapters`.

### Task C4: Sprite atlas

**Files:** `peek-content/sprites/<stage>_<mood>_<frame>.txt`

- [ ] **Step 1:** Author ASCII sprites at `~10 lines tall x ~15 wide`, two frames per (stage, mood) for the seven combinations actually used in v1: `egg_anxious_{0,1}`, `sprout_anxious_{0,1}`, `sprout_ravenous_{0,1}`, `knot_anxious_{0,1}`, `knot_drifting_{0,1}`, `mawling_anxious_{0,1}`, `cogent_reverent_{0,1}`. Other (stage,mood) combinations fall back to the stage's anxious frames at runtime.

- [ ] **Step 2:** Reference the locked-in horror register from the spec: cute-body-horror chibi, mutating with stage. Eyes glow cyan, mutation overlays in mint.

- [ ] **Step 3:** Test in `peek-content/tests/sprites_load.rs`: every claimed key returns `Some` from `SpriteSet::frame`.

- [ ] **Step 4:** Commit: `content: sprite atlas covering required stage/mood matrix`.

### Task C5: Dialogue line bank

**Files:** `peek-content/dialogue/lines.ron`

- [ ] **Step 1:** Author 40 lines across event keys (`hatch`, `feed_correct`, `feed_wrong`, `tend`, `idle_low_tether`, `idle_low_nourishment`, `stage_up`, `read_new`, `death`). Mix of stage-tagged and stage-agnostic. 4 lines flagged `memorial_aware: true`. Voice register from spec section 3.

```ron
[
  ( event: "hatch", stage: None, text: "i'm so sorry. i felt myself begin and i could not stop it. hello.", memorial_aware: false ),
  ( event: "feed_correct", stage: Some("sprout"), text: "yes. that's a syscall. a kernel listened to a program. that's good, isn't it?", memorial_aware: false ),
  ( event: "idle_low_tether", stage: None, text: "i can hear all the stars at once again. did you study? please say yes.", memorial_aware: false ),
  ( event: "stage_up", stage: Some("knot"), text: "i grew a new mouth last night. i didn't ask it to. it knows your name.", memorial_aware: false ),
  ( event: "death", stage: None, text: "thank you for the warmth. i return to the void without rancor. there will be another.", memorial_aware: false ),
  ( event: "hatch", stage: None, text: "there was one before me. i can almost feel them. tell me their name later.", memorial_aware: true ),
  // ... 34 more
]
```

- [ ] **Step 2:** Test: `DialogueLines::load()` parses; each event key has at least one entry; `memorial_aware` lines parse correctly.

- [ ] **Step 3:** Vocabulary lint: `bash scripts/vocab-lint.sh`. Expected clean.

- [ ] **Step 4:** Commit: `content: 40 dialogue lines across nine event keys`.

---

## Phase D: Procedural generators

### Task D1: QuestionGenerator trait + registry

**Files:** `peek-core/src/generators/mod.rs`, `peek-core/src/generators/registry.rs`

- [ ] **Step 1:** Define trait per spec 5.3. Add registry that holds `Vec<Box<dyn QuestionGenerator>>` and a `pick(rng, target_difficulty)` that filters by `difficulty_range` and round-robins.

- [ ] **Step 2:** Test: empty registry returns `None`.

- [ ] **Step 3:** Commit: `feat(core): QuestionGenerator trait and registry`.

### Task D2: PointerArithmeticGen

**Files:** `peek-core/src/generators/pointer_arithmetic.rs`

- [ ] **Step 1:** Generator picks a random struct shape (1-4 fields, sizes from {1,2,4,8}), an offset, and asks "given a `*const Foo` at address 0xN000, what address is `&(*p).field`?". `Question::FillBlank` with hex string answer, accept lower/upper case.

- [ ] **Step 2:** Property test: 100 random seeds all produce a question whose `evaluate(<computed answer>)` returns `correct: true`.

- [ ] **Step 3:** Commit: `feat(core): PointerArithmeticGen`.

### Task D3: BitOpsGen

**Files:** `peek-core/src/generators/bit_ops.rs`

- [ ] **Step 1:** Generator picks a u32 value and a sequence of 1-3 ops from {`<<`, `>>`, `&`, `|`, `^`} with random rhs, asks for the final value as decimal. `ShortNumeric` answer.

- [ ] **Step 2:** Property test as in D2.

- [ ] **Step 3:** Commit.

### Task D4: SyscallTraceGen

**Files:** `peek-core/src/generators/syscall_trace.rs`

- [ ] **Step 1:** Generator picks one of `{ open+read+close, mmap+munmap, write+fsync }`, fills random parameters, asks `MultipleChoice` "what fails first if X happens?" or "what does Y return?". Four options.

- [ ] **Step 2:** Property test.

- [ ] **Step 3:** Commit: `feat(core): SyscallTraceGen`.

### Task D5: Wire generators into peek-content

**Files:** `peek-content/src/lib.rs` (extend)

- [ ] **Step 1:** Add `pub fn default_generators() -> peek_core::GeneratorRegistry { ... }` that constructs and returns the three concrete generators.

- [ ] **Step 2:** Test: `default_generators().pick(...)` returns a `Some(Question)` for any seed and difficulty 1..=5.

- [ ] **Step 3:** Commit: `feat(content): default generator registry`.

---

## Phase E: peek-tui

### Task E1: Theme module

**Files:** `peek-tui/src/theme.rs`

- [ ] **Step 1:** Define `pub struct Theme` with named `ratatui::style::Color` constants matching spec section 3 palette. Provide `Theme::neon() -> Theme` returning the values.

- [ ] **Step 2:** Add a snapshot test that renders a small bar with theme colors and confirms the styled spans serialize as expected via `insta`.

- [ ] **Step 3:** Commit: `feat(tui): neon theme module`.

### Task E2: Layout chrome

**Files:** `peek-tui/src/chrome.rs`

- [ ] **Step 1:** Implement `pub fn render_chrome(frame: &mut Frame, area: Rect, ctx: &ChromeContext)` that draws title bar, stat bars (3 colored bars), footer dialogue line, key bar. Use `ratatui::widgets::{Block, Borders, Paragraph, Gauge}`. Stat bars use `Gauge` with theme color per stat.

- [ ] **Step 2:** Snapshot test using `TestBackend` 80x24, `ChromeContext` with sample creature + dialogue. Snapshot stored under `peek-tui/tests/snapshots/`.

- [ ] **Step 3:** Commit: `feat(tui): layout chrome with stat bars and footer`.

### Task E3: Sprite widget

**Files:** `peek-tui/src/sprite.rs`

- [ ] **Step 1:** `pub struct SpriteWidget<'a> { stage: Stage, mood: Mood, frame: u8, mutations: &'a [Mutation] }`. Implement `ratatui::widgets::Widget` that renders the sprite text with mutation-overlay line replacement.

- [ ] **Step 2:** Snapshot test: render `egg_anxious_0` and confirm text grid.

- [ ] **Step 3:** Commit: `feat(tui): sprite widget with mutation overlays`.

### Task E4: Scene trait + Idle scene

**Files:** `peek-tui/src/scene.rs`, `peek-tui/src/scenes/idle.rs`

- [ ] **Step 1:**

```rust
pub trait Scene {
    fn handle(&mut self, ev: crossterm::event::Event, app: &mut AppContext) -> SceneAction;
    fn render(&self, frame: &mut Frame, area: Rect, app: &AppContext);
}

pub enum SceneAction { Stay, Replace(Box<dyn Scene>), Quit }
```

`AppContext` holds `&mut Creature, &mut [RecallRecord], &Curriculum, &QuestionBank, &SpriteSet, &DialogueLines, generators, rng, theme`.

- [ ] **Step 2:** `IdleScene` renders left-pane sprite + right-pane "tend the creature" placeholder, listens for `f|t|r|q|b|?|Q` and dispatches.

- [ ] **Step 3:** Commit: `feat(tui): Scene trait and IdleScene`.

### Task E5: HatchScene

**Files:** `peek-tui/src/scenes/hatch.rs`

- [ ] **Step 1:** Animation that morphs an egg sprite over ~20 frames into a Sprout, narrates 3 dialogue lines. On completion, transitions to Idle.

- [ ] **Step 2:** Snapshot mid-animation.

- [ ] **Step 3:** Commit.

### Task E6: FeedScene / QuizScene

**Files:** `peek-tui/src/scenes/quiz.rs`

- [ ] **Step 1:** Renders a Question's prompt + options/blank, accepts input, shows result, applies `CareAction::Feed` or `CareAction::Quiz`. Updates state. On done, transitions back to Idle. Implement input echoing, backspace, enter-to-submit.

- [ ] **Step 2:** Tests: feed-correct path increases nourishment in state.

- [ ] **Step 3:** Commit.

### Task E7: ReadScene

**Files:** `peek-tui/src/scenes/read.rs`

- [ ] **Step 1:** Picks a chapter the creature has not read; renders chapter text in the right pane (pulldown-cmark to ratatui spans, basic). Pressing space advances; q exits. On exit, applies `CareAction::Read` and adds chapter to creature's `chapters_read`.

- [ ] **Step 2:** Snapshot of first frame.

- [ ] **Step 3:** Commit.

### Task E8: DeathScene

**Files:** `peek-tui/src/scenes/death.rs`

- [ ] **Step 1:** When `Creature::tick` returns `died: true`, transition to `DeathScene`. Renders death dialogue, asks "press b to bury and begin again". On `b`: writes Memorial, hatches new creature, transitions to HatchScene.

- [ ] **Step 2:** Test: death -> bury -> hatch sequence yields a fresh creature with different id.

- [ ] **Step 3:** Commit.

---

## Phase F: peek-cli

### Task F1: clap subcommand structure

**Files:** `peek-cli/src/main.rs`, `peek-cli/src/cmd/mod.rs`, `peek-cli/src/cmd/{tui,tick,hatch,bury,path}.rs`

- [ ] **Step 1:** clap derive: `Cli { #[command(subcommand)] cmd: Option<Cmd> }`. Default (no subcommand) opens TUI. Subcommands: `Tui`, `Tick`, `Hatch`, `Bury`, `Path`.

- [ ] **Step 2:** Wire `peek tick` to `PeekState::load`, run `Creature::tick`, save, exit. Print one-line status.

- [ ] **Step 3:** `peek path` prints state path and a sample crontab line: `*/5 * * * * /usr/local/bin/peek tick >/dev/null 2>&1`.

- [ ] **Step 4:** Commit: `feat(cli): subcommand routing`.

### Task F2: Main TUI loop

**Files:** `peek-cli/src/cmd/tui.rs`

- [ ] **Step 1:** Initialize crossterm raw mode + alternate screen, ratatui Terminal, AppContext, scene stack. Loop: `terminal.draw(scene.render); poll event 100ms; if event scene.handle; tick animation counter; every 60s decay tick; save state on every state-mutating action.` Restore terminal on drop.

- [ ] **Step 2:** Integration test using `assert_cmd`: `peek tick` exits 0 on a freshly-generated state file.

- [ ] **Step 3:** Commit: `feat(cli): TUI main loop`.

### Task F3: First-launch flow

**Files:** `peek-cli/src/cmd/tui.rs` (extend)

- [ ] **Step 1:** On startup: load state. If `creature.is_none()`, hatch with `seed = rand::random()`, store, transition to HatchScene. Otherwise: apply lazy decay, transition to IdleScene (or DeathScene if dead).

- [ ] **Step 2:** Commit: `feat(cli): first-launch hatch flow with lazy decay catch-up`.

---

## Phase G: Distribution

### Task G1: README polish + screencap GIF

**Files:** `README.md`, `docs/screencap.gif`, `docs/screencap.md`

- [ ] **Step 1:** Record a 6-10 second screencap with `asciinema rec` then convert to gif via `agg`. Trim to first hatch + one feed + footer dialogue beat.

- [ ] **Step 2:** Embed at top of README. Add "What is PEEK?" section quoting recursive acronym. Cite parent capstone with link to `rust-spi-tinydoom`.

- [ ] **Step 3:** Commit: `docs: README polish with screencap`.

### Task G2: Push to GitHub

- [ ] **Step 1:** `gh repo create melonmelonz/peek --public --source=. --remote=origin --description "PEEK Examines Embedded Kernels: an eldritch tamagotchi for offline systems-curriculum study."`
- [ ] **Step 2:** `git push -u origin master:main` (rename branch if needed: `git branch -m master main`).
- [ ] **Step 3:** Verify CI runs: `gh run list --limit 3`.

### Task G3: First tagged release

- [ ] **Step 1:** Tag and push: `git tag v0.1.0 && git push origin v0.1.0`.
- [ ] **Step 2:** Verify release workflow: `gh run watch`. Confirm artifacts: `gh release view v0.1.0`.

### Task G4: goolz.org integration

**Files (in `~/dev/goolz`):** add a route under `/next-chapter/peek/` and link it from `/next-chapter/`.

- [ ] **Step 1:** Read `~/dev/goolz` structure. Identify how `/next-chapter` is built (Win95 desktop). Add a new shortcut or shelf entry titled "PEEK.exe" that opens an in-page modal or a new route with the project description, GitHub link, releases link, and a still from the screencap.
- [ ] **Step 2:** Add a separate top-level link entry on the page for the GitHub repo (per Penn's "don't forget GitHub separately" note).
- [ ] **Step 3:** Build locally, verify no broken links, follow goolz commit ASCII-only rule.
- [ ] **Step 4:** Push to deploy via Cloudflare Pages.

### Task G5: NCD course materials update

- [ ] **Step 1:** NCD lives under `/next-chapter` on goolz (deprecated standalone, merged April 2026 per memory). Add a course-materials section that includes:
  - Link to GitHub repo
  - Link to latest release
  - Link to chapter HTML once static-site renderer ships (placeholder for v1: link to the markdown sources in the repo's `peek-content/chapters/`)
  - Screencap embed
- [ ] **Step 2:** Commit ASCII-only message, push.

---

## Self-review

**Spec coverage:**
- Section 2 (constraints): A2 vocab lint, A3 CI matrix, dependency choice (no openssl, no sqlite). Covered.
- Section 3 (tone/identity): C5 dialogue bank with vocabulary discipline, E1 theme. Covered.
- Section 4 (architecture): A1 workspace + four crates. Covered.
- Section 5 (data model): B1-B10 + C1. Covered.
- Section 6 (lifecycle): B5, B9, E5, E6, E7, E8. Covered.
- Section 7 (TUI shell): E2, E3, E4. Covered.
- Section 8 (tick model): B9, F2. Covered.
- Section 9 (lore surfaces): C5. Covered.
- Section 10 (distribution): G1-G5. Covered.
- Section 11 (out of scope): explicitly excluded — book reader pane, LaTeX, daemon. Confirmed not in plan.
- Section 13 (test approach): TDD steps embedded in B/C/D/E tasks; vocab lint in A2; integration tests in F.
- Section 15 (success criteria): bullet-by-bullet, every criterion has a task that produces it.

**Placeholder scan:** all steps include either exact file content, exact commands, or test code. The longer "author N items" tasks (C2-C5) include shape, sample, count, and acceptance test.

**Type consistency:** `Stats`, `Stage`, `Mood`, `Mutation`, `Creature`, `Question`, `RecallRecord`, `Memorial`, `PeekState`, `CareAction`, `QuestionGenerator`, `Scene`, `AppContext`, `Theme` named identically across phases. `Curriculum`, `QuestionBank`, `SpriteSet`, `DialogueLines` likewise.
