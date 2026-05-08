//! Sprite widget: ASCII art for the creature, with a mutation-overlay band.

use crate::theme::Theme;
use peek_core::{Mood, Mutation, Stage};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

pub struct SpriteWidget<'a> {
    pub stage: Stage,
    pub mood: Mood,
    pub frame: u8,
    pub mutations: &'a [Mutation],
    pub sprite_text: String,
    pub theme: Theme,
}

impl<'a> Widget for SpriteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let style = Style::default().fg(self.theme.accent_cyan).bg(self.theme.bg);
        let pulse = Style::default()
            .fg(self.theme.accent_pink)
            .add_modifier(Modifier::BOLD);

        let lines: Vec<&str> = self.sprite_text.lines().collect();
        let total_h = lines.len() as u16;
        let top_pad = area.height.saturating_sub(total_h) / 2;

        for (i, line) in lines.iter().enumerate() {
            let y = area.y + top_pad + i as u16;
            if y >= area.y + area.height {
                break;
            }
            let chars: Vec<char> = line.chars().collect();
            let total_w = chars.len() as u16;
            let left_pad = area.width.saturating_sub(total_w) / 2;
            for (j, c) in chars.iter().enumerate() {
                let x = area.x + left_pad + j as u16;
                if x >= area.x + area.width {
                    break;
                }
                let s = if matches!(c, 'o' | 'O' | '@' | '*' | '+') {
                    pulse
                } else {
                    style
                };
                let cell = buf.get_mut(x, y);
                cell.set_char(*c);
                cell.set_style(s);
            }
        }

        // Mutation overlay band: print labels under the sprite.
        if !self.mutations.is_empty() {
            let mut s = String::from("[ ");
            let labels: Vec<&str> = self.mutations.iter().map(|m| m.label()).collect();
            s.push_str(&labels.join(", "));
            s.push_str(" ]");
            let y = area.y + top_pad + total_h + 1;
            if y < area.y + area.height {
                let chars: Vec<char> = s.chars().collect();
                let total_w = chars.len() as u16;
                let left_pad = area.width.saturating_sub(total_w) / 2;
                let style = Style::default()
                    .fg(self.theme.accent_mint)
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
    }
}
