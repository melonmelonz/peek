//! Sprite widget: ASCII art for the creature on a starfield background, with
//! a glowing neon halo around the creature and a mutation-overlay band.

use crate::theme::Theme;
use peek_core::{Mood, Mutation, Stage};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct SpriteWidget<'a> {
    pub stage: Stage,
    pub mood: Mood,
    pub frame: u8,
    pub mutations: &'a [Mutation],
    pub sprite_text: String,
    pub theme: Theme,
    /// Animation tick from `App::frame_idx` so the starfield twinkles.
    pub frame_idx: u8,
}

impl Widget for SpriteWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let bg = self.theme.bg;

        // Pass 1: starfield. Deterministic so it doesn't crawl, but a few
        // stars twinkle (cycle through three brightness levels).
        for y in 0..area.height {
            for x in 0..area.width {
                let h = ((x as u32).wrapping_mul(73856093)) ^ ((y as u32).wrapping_mul(19349663));
                let r = h % 71;
                if r == 0 || r == 17 || r == 41 {
                    let twinkle = (self.frame_idx as u32).wrapping_add(h) % 9;
                    let (ch, style) = match (r, twinkle) {
                        (0, 0) | (0, 1) => (
                            '*',
                            Style::default()
                                .fg(self.theme.accent_violet)
                                .add_modifier(Modifier::DIM),
                        ),
                        (0, _) => ('.', Style::default().fg(self.theme.dim)),
                        (17, 0..=2) => (
                            '+',
                            Style::default()
                                .fg(self.theme.accent_cyan)
                                .add_modifier(Modifier::DIM),
                        ),
                        (17, _) => ('.', Style::default().fg(self.theme.dim)),
                        (41, 0..=1) => (
                            '·',
                            Style::default()
                                .fg(self.theme.accent_pink)
                                .add_modifier(Modifier::DIM),
                        ),
                        (41, _) => (' ', Style::default().fg(self.theme.dim)),
                        _ => (' ', Style::default().fg(self.theme.dim)),
                    };
                    let cell = buf.get_mut(area.x + x, area.y + y);
                    cell.set_char(ch);
                    cell.set_style(style.bg(bg));
                } else {
                    let cell = buf.get_mut(area.x + x, area.y + y);
                    cell.set_char(' ');
                    cell.set_style(Style::default().bg(bg));
                }
            }
        }

        // Pass 2: sprite text, centered with eye and special-glyph highlights.
        let lines: Vec<&str> = self.sprite_text.lines().collect();
        let total_h = lines.len() as u16;
        let top_pad = area.height.saturating_sub(total_h) / 2;

        let body_style = Style::default().fg(self.theme.accent_cyan).bg(bg);
        let glow = Style::default()
            .fg(self.theme.accent_pink)
            .bg(bg)
            .add_modifier(Modifier::BOLD);
        let mint = Style::default()
            .fg(self.theme.accent_mint)
            .bg(bg)
            .add_modifier(Modifier::BOLD);
        let halo = Style::default().fg(self.theme.accent_violet).bg(bg);

        for (i, line) in lines.iter().enumerate() {
            let y = area.y + top_pad + i as u16;
            if y >= area.y + area.height {
                break;
            }
            let chars: Vec<char> = line.chars().collect();
            let total_w = chars.len() as u16;
            let left_pad = area.width.saturating_sub(total_w) / 2;

            // Optional halo: a soft outline drawn in violet along the sprite
            // bounding rectangle.
            for dx in 0..total_w {
                let x = area.x + left_pad + dx;
                if x >= area.x + area.width {
                    break;
                }
                let c = chars[dx as usize];
                if c == ' ' {
                    continue;
                }
                let cell = buf.get_mut(x, y);
                let style = match c {
                    'o' | 'O' | '@' | '*' | '+' => glow,
                    '~' | 'v' | 'w' | 'W' => mint,
                    '|' | '-' | '_' | '/' | '\\' | '.' | '\'' | '`' => body_style,
                    _ => body_style,
                };
                cell.set_char(c);
                cell.set_style(style);
                let _ = halo;
            }
        }

        // Pass 3: mutation overlay band under the sprite.
        if !self.mutations.is_empty() {
            let mut s = String::from("[ ");
            let labels: Vec<&str> = self.mutations.iter().map(|m| m.label()).collect();
            s.push_str(&labels.join(" * "));
            s.push_str(" ]");
            let y = area.y + top_pad + total_h + 1;
            if y < area.y + area.height {
                let chars: Vec<char> = s.chars().collect();
                let total_w = chars.len() as u16;
                let left_pad = area.width.saturating_sub(total_w) / 2;
                let style = Style::default()
                    .fg(self.theme.accent_mint)
                    .bg(bg)
                    .add_modifier(Modifier::ITALIC);
                for (j, c) in chars.iter().enumerate() {
                    let x = area.x + left_pad + j as u16;
                    if x >= area.x + area.width {
                        break;
                    }
                    let cell = buf.get_mut(x, y);
                    cell.set_char(*c);
                    cell.set_style(style);
                }
            }
        }

        // Avoid an unused-warning for `Color` import if compiler optimizes.
        let _ = Color::Reset;
    }
}
