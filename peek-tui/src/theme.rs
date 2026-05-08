//! Neon-90s theme palette. Exposed as `ratatui::style::Color` constants.

use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
    pub accent_pink: Color,
    pub accent_cyan: Color,
    pub accent_mint: Color,
    pub accent_violet: Color,
    pub warn: Color,
    pub dim: Color,
}

impl Theme {
    pub const fn neon() -> Self {
        Self {
            fg: Color::Rgb(0xea, 0xea, 0xff),
            bg: Color::Rgb(0x0b, 0x05, 0x1a),
            accent_pink: Color::Rgb(0xff, 0x4d, 0xc7),
            accent_cyan: Color::Rgb(0x4d, 0xe8, 0xff),
            accent_mint: Color::Rgb(0x7c, 0xff, 0xc8),
            accent_violet: Color::Rgb(0xb6, 0x7c, 0xff),
            warn: Color::Rgb(0xff, 0x9f, 0x4d),
            dim: Color::Rgb(0x59, 0x49, 0x82),
        }
    }
}
