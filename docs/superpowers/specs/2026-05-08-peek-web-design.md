# PEEK Web (browser port) — Design Spec

## Goal

A WASM build of PEEK that runs full-fidelity inside `goolz.org/next-chapter`,
embedded in a Win95-style window. The web build raises a real persistent
creature — feed, tend, read, drill, die, bury — using the same domain logic
and curriculum as the native binary. State persists in `localStorage`.

## Non-goals

- No state sync between native and web installs. Each browser is its own
  sandbox, each native install is its own sandbox.
- No backend, no Worker, no Durable Object. Static assets only.
- No reskinning. The web build looks like the TUI: same neon colors, same
  sprites, same layout, same dialogue.

## Architecture

A new crate `peek-web` joins the workspace as a `cdylib` targeting
`wasm32-unknown-unknown`. It depends on `peek-core` (logic) and
`peek-content` (curriculum). It uses [ratzilla](https://github.com/ratatui/ratzilla)
for the ratatui Backend and a wasm-bindgen bridge for input + storage.

Three small refactors land in existing crates:

### 1. `peek-core` storage abstraction

`peek-core` grows a `Storage` trait:

```rust
pub trait Storage {
    fn load_state(&self) -> Result<Option<State>>;
    fn save_state(&self, state: &State) -> Result<()>;
    fn append_memorial(&self, memorial: &Memorial) -> Result<()>;
    fn read_memorials(&self) -> Result<Vec<Memorial>>;
}
```

Native is `FsStorage` (existing logic moves here). Web is `LocalStorage`
(in `peek-web`, uses `web_sys::Storage`). Same RON serde format on both
sides — a state file from the native install is byte-identical to a
localStorage value, which keeps the door open to import/export later.

### 2. `peek-tui` `SceneRunner`

Today, `peek-cli/src/cmd/tui.rs` owns the event loop, the tick cadence,
and scene transitions. Extract this into `peek-tui::scene_runner::SceneRunner`,
generic over the input event type and parameterized by a `Storage`:

```rust
pub struct SceneRunner<S: Storage> { app: App<S>, scene: Box<dyn Scene>, ... }
impl<S: Storage> SceneRunner<S> {
    pub fn handle_input(&mut self, ev: &InputEvent) -> RunnerOutcome;
    pub fn tick(&mut self) -> RunnerOutcome;
    pub fn render(&self, terminal: &mut Terminal<impl Backend>);
}
```

`peek-cli` and `peek-web` both drive this runner — different event source,
different tick driver, same scene logic.

### 3. Input event normalization

A small `InputEvent` enum in `peek-tui` represents the keys PEEK actually
cares about (letter chars, Enter, Backspace, Esc, Space, arrow keys).
crossterm `KeyEvent` and ratzilla `KeyEvent` both adapt into it. Scenes
pattern-match on `InputEvent`, not on the underlying terminal library.

## peek-web crate

```
peek-web/
  Cargo.toml          # cdylib, deps: peek-core, peek-content, peek-tui,
                      # ratzilla, ratatui, wasm-bindgen, web-sys, js-sys,
                      # chrono with wasmbind, rand_chacha
  src/
    lib.rs            # #[wasm_bindgen(start)] — bootstrap
    storage.rs        # LocalStorage: peek_core::Storage impl
    input.rs          # ratzilla::KeyEvent → peek_tui::InputEvent
    runtime.rs        # ratzilla terminal + tick loop + render glue
  index.html          # trunk template, transparent bg
  style.css           # neon overrides on the DOM backend cells
  Trunk.toml
```

### Time

`chrono = { version = "0.4", default-features = false, features = ["clock", "serde", "wasmbind"] }`
plus `js-sys` for `Date::now()`. The existing `Utc::now()` calls work
unchanged in wasm with the `wasmbind` feature.

### Tick cadence

ratzilla's `WebRenderer::draw_web` ties to requestAnimationFrame.
A `setInterval`-style tick at 100ms feeds `runner.tick(...)`; render is
driven by `draw_web` on each rAF.

### Persistence keys

- `peek/state` — RON-serialized `peek_core::State`
- `peek/memorials` — RON-serialized `Vec<Memorial>`

Same schema as native, so a future `peek import-from-web` / `peek export-to-web`
is a 30-line subcommand.

## Build & deployment

### Local dev

```sh
cargo install trunk wasm-bindgen-cli
cd peek-web
trunk serve   # http://localhost:8080, hot reload
```

### CI

A `web` job in `.github/workflows/ci.yml` runs `trunk build --release`
and uploads `peek-web-bundle.tar.gz` as a workflow artifact. On tag pushes,
the same bundle is attached to the GitHub Release alongside the native
binaries.

### Goolz integration

A small `scripts/pull-peek-web.sh` in the goolz repo downloads the latest
release bundle, extracts to `next-chapter/peek/`, and stages the diff for
review:

```sh
GH_TAG=v0.2.0 scripts/pull-peek-web.sh
```

The Win95 window in `next-chapter/index.html` is an iframe that points at
`/next-chapter/peek/`. On click the iframe takes focus so keypresses route
to the WASM runtime, not to the goolz desktop.

## Testing

- All `peek-core` tests continue to pass — storage trait is parametric, no
  behavior change for existing tests; new `FsStorage` tests cover the moved
  filesystem code.
- `peek-tui::scene_runner` gets a `Backend = TestBackend` integration test
  that drives a hatch → idle → feed (correct) → idle sequence and asserts
  on dialogue state and stat deltas.
- A `wasm-bindgen-test` smoke test for `LocalStorage` round-trips a State.
- Manual: `trunk serve`, hatch a creature, refresh, confirm state persists.

## Versioning

This ships as **v0.2.0**. Native binary contract is unchanged; the web
build is additive.

## Risks

- **ratzilla key event coverage** — DomBackend captures keys via DOM events.
  Some characters (e.g., `?`) require shift; verify all PEEK care keys
  (`f`, `t`, `r`, `z`, `b`, `Q`, `?`, `Enter`, `Esc`, `Space`) round-trip
  cleanly.
- **iframe focus** — first-click-to-focus is the standard fix; pre-warm
  with a `tabindex` or programmatic `iframe.contentWindow.focus()` on
  Win95 window activation.
- **trunk in CI** — pin trunk and wasm-bindgen-cli versions in the workflow
  to avoid surprise breakage; include `trunk --version` in the build log.
