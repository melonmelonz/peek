# PEEK Examines Embedded Kernels

An eldritch tamagotchi for the terminal. Raise a small creature by reading
chapters of an embedded systems-and-OS curriculum and answering questions
correctly. Neglect it and it returns to the void.

```
  ___  ___ ___ _  __     examines
 | _ \| __| __| |/ /     embedded
 |  _/| _|| _|| ' <      kernels
 |_|  |___|___|_|\_\     
```

PEEK is single-binary, offline-first, and built for restricted environments
where a coursework game can run on a stock laptop without network or
elevated privileges. The curriculum, question bank, dialogue, and sprites
are baked into the binary at compile time.

## Why this exists

PEEK is the engagement layer for a Slack Next Chapter capstone (May to Aug
2026) on writing a small Rust panel-driver for an SPI display, with DOOM
running on it. The capstone is heavy. PEEK is the daily companion: short
study sessions, a creature that reacts to the work, permadeath that makes
showing up matter.

It is not a hacking tool. The vocabulary is deliberately sanitized; CI
runs `scripts/vocab-lint.sh` to keep the language course-appropriate.

## Install

Prebuilt static-musl binaries are published on GitHub Releases under
[melonmelonz/peek/releases](https://github.com/melonmelonz/peek/releases).
Drop the binary anywhere on `$PATH`.

From source:

```sh
git clone https://github.com/melonmelonz/peek
cd peek
cargo install --path peek-cli
```

## Use

```sh
peek            # launch the TUI
peek tick       # apply lazy decay (cron this every 5 minutes)
peek hatch      # roll a fresh egg (only if no creature exists)
peek bury       # bury and reseed if the creature is gone
peek path       # print state and memorial paths
```

State lives at `~/.local/state/peek/state.ron`.
Memorials append to `~/.local/state/peek/memorials.ron`.

### Care keys, in the TUI

| key | action                                       |
|-----|----------------------------------------------|
| `f` | feed it a question (banked, spaced repetition) |
| `t` | tend (steady the tether)                     |
| `r` | read a chapter together                      |
| `z` | drill (procedurally generated questions)     |
| `?` | help                                         |
| `Q` | quit                                         |

## How it works

Three stats decay independently with a half-life model:

- **nourishment** — fed by correct answers
- **tether**     — fed by tending and by reading chapters together
- **lucidity**   — fed by completing chapters and by drilling generated questions

The creature progresses through six stages:
**Egg → Sprout → Knot → Mawling → Conduit → Cogent.**
At any stage it can die if all three stats hit zero. Death is permanent.
A memorial is appended; a fresh egg arrives.

There is no daemon. Decay is computed lazily from wall time on each launch
and on each `peek tick`. Cron is the recommended scheduler:

```cron
*/5 * * * * /usr/local/bin/peek tick >/dev/null 2>&1
```

## Workspace layout

```
peek-core/      # domain types, decay, care, recall (SM-2 lite), generators
peek-content/   # embedded curriculum, question bank, dialogue, sprites
peek-tui/       # ratatui scenes, neon chrome, sprite renderer
peek-cli/       # clap entrypoint + subcommands
```

Roughly 40 unit tests in `peek-core` and 5 in `peek-content`. Generators
are exercised over many seeds to catch off-by-one errors in things like
struct field offsets.

## Aesthetic

Neon-90s. Pink, cyan, mint, violet on a near-black violet background. Block
characters with a shimmering leading edge for the stat bars. Hand-authored
ASCII for the title and creature sprites. Twinkling starfield behind the
creature. The dialogue voice is anxious-eldritch and chibi-cute, not
threatening.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT`
and `LICENSE-APACHE`.
