use crate::crosswords::grid::GridSquare;
use crate::crosswords::Column;
use crate::crosswords::Row;
use bitflags::bitflags;
use colors::{AnsiColor, NamedColor};
use std::sync::Arc;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Flags: u16 {
        const INVERSE                   = 0b0000_0000_0000_0001;
        const BOLD                      = 0b0000_0000_0000_0010;
        const ITALIC                    = 0b0000_0000_0000_0100;
        const BOLD_ITALIC               = 0b0000_0000_0000_0110;
        const UNDERLINE                 = 0b0000_0000_0000_1000;
        const WRAPLINE                  = 0b0000_0000_0001_0000;
        const WIDE_CHAR                 = 0b0000_0000_0010_0000;
        const WIDE_CHAR_SPACER          = 0b0000_0000_0100_0000;
        const DIM                       = 0b0000_0000_1000_0000;
        const DIM_BOLD                  = 0b0000_0000_1000_0010;
        const HIDDEN                    = 0b0000_0001_0000_0000;
        const STRIKEOUT                 = 0b0000_0010_0000_0000;
        const LEADING_WIDE_CHAR_SPACER  = 0b0000_0100_0000_0000;
        const DOUBLE_UNDERLINE          = 0b0000_1000_0000_0000;
        const UNDERCURL                 = 0b0001_0000_0000_0000;
        const DOTTED_UNDERLINE          = 0b0010_0000_0000_0000;
        const DASHED_UNDERLINE          = 0b0100_0000_0000_0000;
        // const ALL_UNDERLINES            = Self::UNDERLINE.bits | Self::DOUBLE_UNDERLINE.bits
        //                                 | Self::UNDERCURL.bits | Self::DOTTED_UNDERLINE.bits
        //                                 | Self::DASHED_UNDERLINE.bits;
    }
}

/// Dynamically allocated cell content.
///
/// This storage is reserved for cell attributes which are rarely set. This allows reducing the
/// allocation required ahead of time for every cell, with some additional overhead when the extra
/// storage is actually required.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct CellExtra {
    zerowidth: Vec<char>,
    // underline_color: Option<colors::AnsiColor>,

    // hyperlink: Option<Hyperlink>,
}

/// Content and attributes of a single cell in the terminal grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Square {
    pub c: char,
    pub fg: AnsiColor,
    pub bg: AnsiColor,
    pub extra: Option<Arc<CellExtra>>,
    pub flags: Flags,
}

impl Default for Square {
    #[inline]
    fn default() -> Square {
        Square {
            c: ' ',
            bg: AnsiColor::Named(NamedColor::Black),
            fg: AnsiColor::Named(NamedColor::Foreground),
            extra: None,
            flags: Flags::empty(),
        }
    }
}

impl Square {
    #[allow(dead_code)]
    #[inline]
    pub fn zerowidth(&self) -> Option<&[char]> {
        self.extra.as_ref().map(|extra| extra.zerowidth.as_slice())
    }

    /// Write a new zerowidth character to this cell.
    #[inline]
    pub fn push_zerowidth(&mut self, character: char) {
        let extra = self.extra.get_or_insert(Default::default());
        Arc::make_mut(extra).zerowidth.push(character);
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn clear_wide(&mut self) {
        self.flags.remove(Flags::WIDE_CHAR);
        if let Some(extra) = self.extra.as_mut() {
            Arc::make_mut(extra).zerowidth = Vec::new();
        }
        self.c = ' ';
    }
}

impl GridSquare for Square {
    #[inline]
    fn is_empty(&self) -> bool {
        (self.c == ' ' || self.c == '\t')
            && !self.flags.intersects(
                Flags::INVERSE
                    // | Flags::ALL_UNDERLINES
                    | Flags::STRIKEOUT
                    | Flags::WRAPLINE
                    | Flags::WIDE_CHAR_SPACER
                    | Flags::LEADING_WIDE_CHAR_SPACER,
            )
            && self.extra.as_ref().map(|extra| extra.zerowidth.is_empty()) != Some(false)
    }

    #[inline]
    fn reset(&mut self, template: &Self) {
        *self = Square {
            bg: template.bg,
            ..Square::default()
        };
    }

    #[inline]
    fn flags(&self) -> &Flags {
        &self.flags
    }

    #[inline]
    fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }
}

pub trait LineLength {
    /// Calculate the occupied line length.
    fn line_length(&self) -> Column;
}

impl LineLength for Row<Square> {
    fn line_length(&self) -> Column {
        let mut length = Column(0);

        if self[Column(self.len() - 1)].flags.contains(Flags::WRAPLINE) {
            return Column(self.len());
        }

        for (index, cell) in self[..].iter().rev().enumerate() {
            if cell.c != ' '
                || cell.extra.as_ref().map(|extra| extra.zerowidth.is_empty())
                    == Some(false)
            {
                length = Column(self.len() - index);
                break;
            }
        }

        length
    }
}

pub trait ResetDiscriminant<T> {
    /// Value based on which equality for the reset will be determined.
    fn discriminant(&self) -> T;
}

impl<T: Copy> ResetDiscriminant<T> for T {
    fn discriminant(&self) -> T {
        *self
    }
}

impl ResetDiscriminant<AnsiColor> for Square {
    fn discriminant(&self) -> AnsiColor {
        self.bg
    }
}