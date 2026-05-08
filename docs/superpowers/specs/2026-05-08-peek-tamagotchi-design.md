# PEEK tamagotchi v1 design

**Status:** draft, awaiting Penn's review
**Date:** 2026-05-08
**Author:** Penn Porterfield + Claude
**Project:** PEEK Examines Embedded Kernels (capstone, Slack Next Chapter, May–Aug 2026)
**Repo (planned):** github.com/melonmelonz/peek

## 1. Summary

A neon-90s TUI tamagotchi that lives in a zellij pane. The creature is a tiny eldritch horror that knows it is a tiny eldritch horror, embarrassed about it, anxious to please, with a body that mutates in disturbing ways and dialogue that occasionally slips into vast cosmic indifference. You raise it by walking through PEEK's embedded systems-curriculum: reading chapters with it, answering questions correctly, and visiting often enough that it stays tethered to this plane.

This is the engagement layer for PEEK proper. The same Cargo workspace will later house the book reader, the LaTeX/PDF render path, and the static-site bundle described in `docs/roadmap/roadmap.pdf` of the parent capstone. v1 ships only the tamagotchi mode and the question/recall engine that the book reader will eventually consume.

## 2. Constraints (carried over from PEEK roadmap)

These are non-negotiable in the parent capstone and apply here:

- **Static musl binary.** No dynamic linking, no system libraries beyond libc.
- **No network.** Naturally air-gapped. Curriculum and question bank are embedded at compile time.
- **No crypto.** Skips an entire vocabulary of facility-review friction.
- **No game payload.** Discoverable and downloadable freely.
- **No exploitation vocabulary.** "Engineering, not hacking." No "exploit," "crack," "breach," "expose-vuln," etc. in lore, copy, or commit messages. The eldritch flavor speaks in *nourishment*, *coherence*, *void*, *tether*, not predation.
- **Reentry-classroom audience.** Copy must read as study material, not edginess for its own sake. Horror tone is bittersweet, not nihilist.

## 3. Tone and identity

Locked register: **cosmic-dread setting + cute-body-horror creature + anxious-eldritch dialogue.** A tiny creature in a vast indifferent void, mutating cutely, apologizing for its cosmic insights.

**Voice samples (for copy authoring):**

- on hatch: "i'm so sorry. i felt myself begin and i could not stop it. hello."
- on feeding (low difficulty correct): "yes, that's a syscall. i'm proud. a kernel listened to a program. that's… that's good, isn't it?"
- on neglect (low tether): "i can hear all the stars at once again. did you study? please say yes."
- on stage transition: "i grew a new mouth last night. i didn't ask it to. it knows your name."
- on death: "thank you for the warmth. i return to the void without rancor. there will be another."

**Palette (named, hex):**

```
void          #050308   background
hi-vis        #fff95c   primary stat bars, key affordances
magenta       #ff2bd6   warnings, low stats, mutation reveal
cyan          #00e0ff   interactive accents, cursor, prompt
chibi-pink    #ff7ab8   creature body fill
sickly-mint   #7cfc9f   mutation highlights, "wrong" colors
bone          #fff5e0   chapter text, recall prompts
ash           #555555   muted UI, completed states
```

## 4. Architecture

Cargo workspace with four crates:

| Crate | Kind | Purpose |
|-------|------|---------|
| `peek-core` | lib | Domain types and pure logic. No UI deps. Decay tick, recall scheduling, hatching RNG, save/load. |
| `peek-content` | lib | Embedded curriculum: markdown chapters, question-bank RON, sprite-art constants. Read-only. Built with `rust-embed` at compile time. Same crate later feeds LaTeX/site renderers. |
| `peek-tui` | lib | ratatui scenes and widgets. No I/O beyond stdout/stdin. `Scene` trait for Hatch / Idle / Read / Quiz / Mutate / Death. |
| `peek-cli` | bin | Static-musl binary. Subcommands: `peek` (default TUI), `peek tick`, `peek hatch`, `peek bury`, `peek path`. |

**Why split this way:** `peek-core` is the canonical engine PEEK already promises: one source, multiple consumers. The TUI is one consumer; the LaTeX/PDF book pipeline (week 11 of the capstone roadmap) is the second; the static-site bundle is the third. `peek-content` separates *what we teach* from *how we teach it*, so curriculum edits don't touch logic or rendering.

**Why ratatui:** active community as of 2026, idiomatic Rust, custom-widget friendly, crossterm backend works on musl. Sprite animation and stat bars are within its ergonomic range.

**Why no daemon:** real-time feel without daemon complexity is achievable with lazy-tick on launch + an in-TUI tick thread + an optional `peek tick` for cron/systemd users who want decay to advance while the TUI is closed. Facility deployment forbids user daemons in some venues; cron does not.

## 5. Data model

### 5.1 Creature

```rust
pub struct Creature {
    pub id: Uuid,                  // for memorial linkage on death
    pub seed: u64,                 // hatch RNG seed; everything below derives from this
    pub true_name: String,         // generated from seed; cannot be changed
    pub stage: Stage,              // Egg | Sprout | Knot | Mawling | Conduit | Cogent
    pub mood: Mood,                // Anxious | Lucid | Drifting | Ravenous | Reverent
    pub mutations: Vec<Mutation>,  // accumulated visible mutations
    pub stats: Stats,              // see 5.2
    pub born_at: DateTime<Utc>,
    pub last_tick: DateTime<Utc>,
    pub correct_recalls: u32,
    pub chapters_read: BTreeSet<ChapterId>,
}
```

`Stage` is `Egg | Sprout | Knot | Mawling | Conduit | Cogent`, indices 0..=5. Each stage unlocks new mutations and shifts the dialogue register slightly more cosmic.

`Mood` is `Anxious | Lucid | Drifting | Ravenous | Reverent`. It is soft state, derived from stats and recent events on each tick. Anxious = default; Drifting = low tether; Ravenous = low nourishment; Lucid = high lucidity; Reverent = post-stage-up window. Mood selects sprite frames and biases dialogue line pulls.

`Mutation` is a small enum naming a sprite overlay: `ExtraEye | ThirdMouth | Tendril { count: u8 } | InvertedSpiral | NoneAtAll`. Mutations are rolled when the creature stages up, drawn from the seed-derived RNG. v1 ships ~6 mutation kinds. Each mutation knows which sprite lines it overlays.

### 5.2 Stats and decay

Three needs, three independent decay curves. All restored by player action; all decay on the wall clock.

```rust
pub struct Stats {
    pub nourishment: f32, // 0.0..=1.0   restored: correct recalls, chapter completions
    pub tether: f32,      // 0.0..=1.0   restored: visits, "tend" action, naming events
    pub lucidity: f32,    // 0.0..=1.0   restored: NEW chapter content only (not re-recall)
}
```

Decay rates (initial tuning, expect to adjust):

- **Nourishment:** half-life 36 hours. Falls fastest; primary care driver.
- **Tether:** half-life 72 hours. Resistant to short absences; punishes weeklong silence.
- **Lucidity:** half-life 60 hours. Cannot be farmed by repeating known questions, which forces curriculum advancement.

If any stat hits 0.0 and stays at 0.0 for 24 hours, the creature dies. Death is permanent for that creature. State writes a `Memorial` entry to `memorials.ron` and rolls a fresh egg.

### 5.3 Lesson and Question

```rust
pub struct ChapterId(pub &'static str);     // e.g. "ch03-syscalls"
pub struct Question {
    pub id: QuestionId,
    pub chapter: ChapterId,
    pub difficulty: Difficulty,             // 1..=5
    pub kind: QuestionKind,
    pub explanation: String,                // shown after attempt
    pub tags: Vec<&'static str>,
}
pub enum QuestionKind {
    MultipleChoice { prompt: String, options: Vec<String>, correct: u8 },
    FillBlank      { prompt: String, blank: String, accept: Vec<String> },
    ShortNumeric   { prompt: String, accept_range: RangeInclusive<f64> },
    TraceProgram   { source: String, expected_output: String }, // code snippet, ask "what prints?"
}
```

Two sources of questions:

1. **Hand-authored bank**: RON file at `peek-content/questions/bank.ron`. ~50–80 questions for v1, tagged by chapter and difficulty.
2. **Procedural generators**: three generators in v1, each implementing:

```rust
pub trait QuestionGenerator: Send + Sync {
    fn chapter(&self) -> ChapterId;
    fn difficulty_range(&self) -> RangeInclusive<u8>;
    fn generate(&self, rng: &mut dyn RngCore, target: u8) -> Question;
}
```

v1 generators:

- `PointerArithmeticGen` (chapter: pointers/memory): generates "what does `p + n` point to given struct layout X?" with random sizes within sensible ranges.
- `BitOpsGen` (chapter: representation): random masks, shifts, signed/unsigned conversions; "what does this expression evaluate to?"
- `SyscallTraceGen` (chapter: syscalls): picks a small syscall sequence (`open`, `read`, `write`, `close`; or `mmap`, `munmap`) with random parameters, asks for the kernel-visible effect or the return value on a given failure mode.

Generators are seeded so the same generator state produces the same question (testable, replayable).

### 5.4 Recall schedule

SM-2 lite (SuperMemo 2). Per question we track:

```rust
pub struct RecallRecord {
    pub question: QuestionId,
    pub last_seen: DateTime<Utc>,
    pub interval_hours: f32,
    pub ease: f32,            // 1.3..=2.5
    pub streak: u16,
    pub next_due: DateTime<Utc>,
}
```

After each attempt, ease and interval update by SM-2 rules (correct = grow interval by ease; incorrect = reset interval to 1h, drop ease by 0.2, floor 1.3). Quiz mode pulls due records first, then introduces a new question if the creature has lucidity capacity for it.

### 5.5 State on disk

Single file at `$XDG_STATE_HOME/peek/state.ron` (falls back to `$HOME/.local/state/peek/state.ron`). RON for human-readable diffability. Schema:

```rust
pub struct PeekState {
    pub schema_version: u32,         // currently 1
    pub creature: Option<Creature>,  // None if between hatches
    pub recall: Vec<RecallRecord>,
    pub recent_dialogue: VecDeque<DialogueEvent>, // ring buffer of last 64 lines for footer scroll
    pub install_id: Uuid,            // anonymous local id, never transmitted
}
```

Plus `memorials.ron` next to it, append-only:

```rust
pub struct Memorial {
    pub creature_id: Uuid,
    pub true_name: String,
    pub born_at: DateTime<Utc>,
    pub died_at: DateTime<Utc>,
    pub final_stage: Stage,
    pub chapters_read: u32,
}
```

Memorials feed into the new creature's occasional dialogue ("there was one before me. i can almost feel them.").

## 6. Care loop and lifecycle

```
egg laid (first launch)
   ↓
hatch (random seed; egg cracks over first session)
   ↓
sprout: basic sprite, low stats, simple dialogue
   ↓        feed / tend / read enough → stage up
knot: first mutations appear, voice gets stranger
   ↓
mawling: recall difficulty unlocks 3+
   ↓
conduit: recall difficulty unlocks 5; cosmic dialogue dominates
   ↓
cogent: final stage; creature begins offering its own questions back to you
   ↓
death (any stat at 0 for 24h) → memorial written → new egg next launch
```

**Player actions (key bindings):**

- `f` feed. Present a due recall question. Correct lifts nourishment, with a small lucidity bump if the question is new. Wrong drops tether a touch (it's hurt).
- `t` tend. Petting and talking. Tether up, no other effect. Free, low cooldown.
- `r` read. Pull a chapter excerpt. Reading a chapter the creature hasn't seen lifts lucidity. Re-reading gives a small tether bump only.
- `q` quiz. Same as feed but pulls a new (un-recalled) question. Use it when you want to advance the curriculum rather than maintain.
- `b` bury (only available when dead). Writes memorial, rolls new egg.
- `?`: help.
- `Q`: quit.

## 7. TUI shell

Single window, three regions, fits ≥80×24:

```
┌─ PEEK • Examines Embedded Kernels ─────── N ▓▓▓░░  T ▓▓░░░  L ▓░░░░  ── stage: knot ─┐
│                                          │                                            │
│        ╔═══════════════╗                 │  > what does mmap return on failure?       │
│        ║   [sprite,    ║                 │                                            │
│        ║    animated]  ║                 │   [a] -1, errno set                        │
│        ║               ║                 │   [b] NULL                                 │
│        ╚═══════════════╝                 │   [c] MAP_FAILED                           │
│                                          │                                            │
│        name: vex'kael                    │   answer: _                                │
│        stage: knot                       │                                            │
│        mood: anxious                     │                                            │
│                                          │                                            │
├──────────────────────────────────────────┴────────────────────────────────────────────┤
│ "i can hear all the stars at once again. did you study? please say yes."             │
├───────────────────────────────────────────────────────────────────────────────────────┤
│ [f]eed  [t]end  [r]ead  [q]uiz  [b]ury  [?]help  [Q]uit                              │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

Top bar: title left, stat bars and stage right.
Left pane: creature sprite (animated, ~5fps), name/stage/mood readout.
Right pane: the active scene's prompt (chapter excerpt, recall question, dialogue beat).
Footer: most recent dialogue line, scrolls back via `[`/`]`.
Key bar: action affordances.

Sprites are plain `&'static str` constants per stage and mood, with optional 2–3 frames for ambient animation. ratatui `Paragraph` with styled spans paints them; a 200ms animation tick swaps frames. Mutations layer over the base sprite by replacing specific lines (e.g., the eye row gets a "third eye" overlay).

## 8. Tick model

Three independent ticks:

1. **Decay tick**: runs once per minute while the TUI is open; also runs once on launch (catches up wall-clock time since `last_tick`); also runs when `peek tick` is invoked from cron.
2. **Animation tick**: 200ms while TUI is open, swaps sprite frames.
3. **Dialogue tick**: every 30–120 seconds (jittered) while idle, the creature emits a contextual line into the footer ring buffer.

`peek tick` exits cleanly without opening a TUI. Suggested user crontab line gets printed by `peek path`.

## 9. Lore and writing surfaces

Three places where the eldritch tone lives in copy:

- **Sprite/mood pairings**: visual identity. Authored manually.
- **Dialogue lines**: keyed by event (hatch, feed-correct, feed-wrong, tend, idle-low-tether, stage-up, death) and by stage. ~200 lines for v1, in a RON file. Kept short, varied cadence, never edgy for its own sake. Sample lines reviewed against the "engineering, not hacking" rule.
- **Memorial integration**: a fraction of dialogue lines have a `memorial_aware: true` flag and only fire if a memorial exists. They reference the prior creature obliquely.

Writing target: somewhere between Junji Ito chibi panels and Welcome to Night Vale weather reports. Bittersweet over nihilist. The creature is a kid asking questions.

## 10. Distribution

Three deliverables, all linked from the eventual goolz.org/next-chapter section:

1. **GitHub repo:** `github.com/melonmelonz/peek`. Public from day one. README with screencap GIF, install instructions, build-from-source steps, license. Tagged releases trigger a GitHub Actions matrix build that produces:
   - `peek-x86_64-unknown-linux-musl` (statically linked)
   - `peek-aarch64-unknown-linux-musl` (statically linked)
   Attached to the release with sha256 sums. No installer, no package; just the binary.

2. **goolz.org integration:** under `/next-chapter/peek/`, a page links the repo, the latest release downloads, and (later) the static-site rendering of the curriculum. Distinct from the repo link; both surfaces live on the index page so visitors can pick.

3. **NCD course materials:** the same goolz section serves as the NCD entry. v1 lists "the tamagotchi" with download + screencap. Later versions add the chapter index and the LaTeX/PDF book.

License: dual MIT / Apache-2.0 for code (Rust ecosystem default). Curriculum text under CC BY 4.0. Both noted in `LICENSE` and README.

## 11. Out of scope (YAGNI for v1)

- Book reader pane (stubbed widget says "PEEK READER comes online at stage 3" or similar in-lore message).
- LaTeX/PDF and static-site renderers (slot exists in `peek-content`; no consumers).
- Daemon mode (`peekd`).
- Multiple concurrent creatures.
- State migration tooling beyond the schema_version field.
- Localization.
- Audio / sound effects.
- Save-state cloud sync. (Forbidden by the no-network rule.)
- Custom user-authored question packs at runtime.

## 12. Risks

| Risk | Mitigation |
|------|------------|
| Tone slips into edgy/exploit-flavored | Vocabulary check before any release: grep dialogue + UI strings for blocked words. Reviewed against PEEK roadmap's "engineering, not hacking" rule. |
| Tamagotchi work eats roadmap weeks needed for kernel module | Fix v1 scope to what this doc enumerates. Defer book reader until week 11 of the parent capstone, as the roadmap already plans. |
| Spaced-recall tuning feels punishing | All decay rates and SM-2 parameters live in a single `Tuning` struct. Adjustable without code changes via a debug subcommand `peek tune` (dev-only feature flag). |
| Static-musl build breaks on a transitive C dep | Pin to known-good crates. ratatui + crossterm are pure-Rust. rust-embed is pure-Rust. No sqlite, no openssl, no native dependencies in v1. |
| Question bank lacks variety after a few sessions | Three procedural generators provide effectively-infinite questions; bank is for breadth, generators for depth. Add new generators incrementally without schema changes. |

## 13. Test approach

- **`peek-core` unit tests:** decay math, recall scheduling round-trip, RNG-seeded hatch determinism (same seed → same creature traits), save/load round-trip on RON.
- **`peek-content` data tests:** every question in the bank parses; every chapter referenced in the bank exists; sprite art constants are valid widths.
- **`peek-tui` snapshot tests:** ratatui `TestBackend` + `insta` snapshots for each scene at known states.
- **`peek-cli` integration tests:** `assert_cmd` for subcommands; `peek tick` advances state and exits 0; `peek` exits cleanly on `Q`.
- **Property tests:** each procedural generator emits well-formed `Question`s for any seed; SM-2 update never produces NaN ease.
- **Vocabulary lint:** simple grep-based test that fails CI if blocked-word list appears in any RON or rust string literal.

## 14. Build, repo layout

```
peek/
├── Cargo.toml                       # workspace
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── docs/
│   ├── superpowers/specs/           # this doc and successors
│   └── roadmap/                     # later, mirroring rust-spi-tinydoom style
├── peek-core/
├── peek-content/
│   ├── chapters/                    # markdown
│   ├── questions/bank.ron
│   └── sprites/
├── peek-tui/
└── peek-cli/
```

CI: GitHub Actions. Matrix: stable Rust, x86_64-musl + aarch64-musl. Steps: fmt, clippy --deny warnings, test, build release musl, vocabulary lint. Release tag publishes binaries.

## 15. Success criteria for v1

- `cargo run` opens the TUI in a zellij pane, fits 80×24, renders a creature.
- A first-launch hatch sequence plays out and writes a creature to state.
- Feed/tend/read/quiz actions all change stats correctly and persist across runs.
- At least three stages reachable in test (creature can be force-advanced via dev-only flag).
- Death + memorial + rehatch flow works end to end.
- Quiz pulls due recall questions; new questions advance lucidity; SM-2 schedule updates.
- All three procedural generators produce questions that pass the property tests.
- Static-musl release binary runs on a stock Linux box without dynamic linker errors.
- README has a screencap. Vocabulary lint passes.
- Repo public on GitHub. goolz.org/next-chapter section links it.

---

## Open items for Penn

1. Are you OK with the three-stat decay model (nourishment / tether / lucidity)? It adds one more axis than classic tamagotchi but enforces "you can't farm easy questions to keep it alive."
2. Are stage names (Egg / Sprout / Knot / Mawling / Conduit / Cogent) tonally on? Easy to rename; let me know.
3. Should `peek` also act as the future book-reader entry point now (TUI launches with creature pane + chapter pane), or keep v1 creature-only and add the chapter pane in v2?
4. GitHub org for the repo: confirm `melonmelonz/peek` (matches your other capstone repos) vs anything else.
