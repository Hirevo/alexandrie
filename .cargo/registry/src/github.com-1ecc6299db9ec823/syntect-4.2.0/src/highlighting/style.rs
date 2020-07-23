// Code based on https://github.com/defuz/sublimate/blob/master/src/core/syntax/style.rs
// released under the MIT license by @defuz
use bitflags::bitflags;

/// The foreground, background and font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Style {
    /// Foreground color.
    pub foreground: Color,
    /// Background color.
    pub background: Color,
    /// Style of the font.
    pub font_style: FontStyle,
}

/// A change to a `Style` applied incrementally by a theme rule.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleModifier {
    /// Foreground color.
    pub foreground: Option<Color>,
    /// Background color.
    pub background: Option<Color>,
    /// Style of the font.
    pub font_style: Option<FontStyle>,
}

/// RGBA color, these numbers come directly from the theme so
/// for now you might have to do your own color space conversion if you are outputting
/// a different color space from the theme. This can be a problem because some Sublime
/// themes use sRGB and some don't. This is specified in an attribute syntect doesn't parse yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
    /// Alpha component
    pub a: u8,
}

bitflags! {
    /// This can be a combination of `BOLD`, `UNDERLINE` and `ITALIC`
    #[derive(Serialize, Deserialize)]
    pub struct FontStyle: u8 {
        /// Bold font style
        const BOLD = 1;
        /// Underline font style
        const UNDERLINE = 2;
        /// Italic font style
        const ITALIC = 4;
    }
}


impl Color {
    /// Black color (`#000000`)
    pub const BLACK: Color = Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0xFF,
    };

    /// White color (`#FFFFFF`)
    pub const WHITE: Color = Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
        a: 0xFF,
    };
}

impl Style {
    /// Applies a change to this style, yielding a new changed style
    pub fn apply(&self, modifier: StyleModifier) -> Style {
        Style {
            foreground: modifier.foreground.unwrap_or(self.foreground),
            background: modifier.background.unwrap_or(self.background),
            font_style: modifier.font_style.unwrap_or(self.font_style),
        }
    }
}

impl Default for Style {
    fn default() -> Style {
        Style {
            foreground: Color::BLACK,
            background: Color::WHITE,
            font_style: FontStyle::empty(),
        }
    }
}

impl StyleModifier {
    /// Applies the other modifier to this one, creating a new modifier.
    /// Values in `other` are preferred.
    pub fn apply(&self, other: StyleModifier) -> StyleModifier {
        StyleModifier {
            foreground: other.foreground.or(self.foreground),
            background: other.background.or(self.background),
            font_style: other.font_style.or(self.font_style),
        }
    }
}
