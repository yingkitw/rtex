//! TeX glue — flexible spacing with stretch and shrink.
//!
//! Glue is TeX's fundamental spacing primitive: a width plus optional
//! stretch and shrink components. Infinite glue orders (`fil`, `fill`,
//! `filll`) participate in space distribution during line and page building.

use super::dimensions::Dimension;

/// Infinite glue unit order (TeX `fil`, `fill`, `filll`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InfiniteUnit {
    /// First-order infinite glue (`fil`).
    Fil = 1,
    /// Second-order infinite glue (`fill`).
    Fill = 2,
    /// Third-order infinite glue (`filll`).
    Filll = 3,
}

/// Stretch component — finite dimension or infinite glue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stretch {
    None,
    Finite(Dimension),
    Infinite { amount: i32, unit: InfiniteUnit },
}

/// A TeX glue specification: natural width + stretch + shrink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glue {
    pub width: Dimension,
    pub stretch: Stretch,
    pub shrink: Dimension,
}

impl Glue {
    /// Rigid glue with no stretch or shrink.
    pub fn natural(width: Dimension) -> Self {
        Self {
            width,
            stretch: Stretch::None,
            shrink: Dimension::from_sp(0),
        }
    }

    /// Zero-width glue.
    pub fn zero() -> Self {
        Self::natural(Dimension::from_sp(0))
    }

    /// `\hfill` — `0pt plus 1fil minus 0pt`.
    pub fn hfill() -> Self {
        Self {
            width: Dimension::from_sp(0),
            stretch: Stretch::Infinite {
                amount: 1,
                unit: InfiniteUnit::Fil,
            },
            shrink: Dimension::from_sp(0),
        }
    }

    /// `\hfil` — `0pt plus 1fil`.
    pub fn hfil() -> Self {
        Self::hfill()
    }

    /// Parse a TeX glue specification.
    ///
    /// Supported forms:
    /// - `10pt`
    /// - `10pt plus 2pt`
    /// - `10pt minus 1pt`
    /// - `10pt plus 1fil minus 2pt`
    /// - `0pt plus 1fill`
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        let (width_part, rest) = split_keyword(s, " plus ")?;
        let width = Dimension::parse(width_part)?;

        let mut stretch = Stretch::None;
        let mut shrink = Dimension::from_sp(0);

        if let Some(r) = rest {
            let (stretch_part, after_stretch) = split_keyword(r, " minus ")?;
            stretch = parse_stretch(stretch_part)?;
            if let Some(shrink_part) = after_stretch {
                shrink = Dimension::parse(shrink_part)?;
            }
        }

        Some(Self {
            width,
            stretch,
            shrink,
        })
    }

    /// Resolve glue to a concrete width given `ratio` in `[0, 1]` for finite stretch.
    ///
    /// Infinite glue returns its natural width here; a full line-breaking pass
    /// would distribute infinite units separately.
    pub fn resolve_finite(&self, ratio: f64) -> Dimension {
        let extra = match self.stretch {
            Stretch::None => 0,
            Stretch::Finite(d) => (d.sp() as f64 * ratio.clamp(0.0, 1.0)).round() as i64,
            Stretch::Infinite { .. } => 0,
        };
        let shrink_amt = (self.shrink.sp() as f64 * (1.0 - ratio).clamp(0.0, 1.0)).round() as i64;
        Dimension::from_sp(self.width.sp() + extra - shrink_amt)
    }

    /// Whether this glue can absorb infinite stretch (`fil`/`fill`/`filll`).
    pub fn is_infinite(&self) -> bool {
        matches!(self.stretch, Stretch::Infinite { .. })
    }
}

fn split_keyword<'a>(s: &'a str, keyword: &str) -> Option<(&'a str, Option<&'a str>)> {
    match s.find(keyword) {
        Some(idx) => {
            let before = s[..idx].trim();
            let after = s[idx + keyword.len()..].trim();
            if after.is_empty() {
                Some((before, None))
            } else {
                Some((before, Some(after)))
            }
        }
        None => Some((s.trim(), None)),
    }
}

fn parse_stretch(s: &str) -> Option<Stretch> {
    let s = s.trim();
    if s.is_empty() {
        return Some(Stretch::None);
    }

    if let Some((amount, unit)) = parse_infinite_unit(s) {
        return Some(Stretch::Infinite { amount, unit });
    }

    Dimension::parse(s).map(Stretch::Finite)
}

fn parse_infinite_unit(s: &str) -> Option<(i32, InfiniteUnit)> {
    let s = s.trim();
    for (suffix, unit) in [
        ("filll", InfiniteUnit::Filll),
        ("fill", InfiniteUnit::Fill),
        ("fil", InfiniteUnit::Fil),
    ] {
        if let Some(num) = s.strip_suffix(suffix) {
            let amount: i32 = if num.is_empty() {
                1
            } else {
                num.parse().ok()?
            };
            return Some((amount, unit));
        }
    }
    None
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::tex::dimensions::PT_TO_SP;

    #[test]
    fn parse_rigid_glue() {
        let g = Glue::parse("10pt").unwrap();
        assert_eq!(g.width.pt(), 10.0);
        assert_eq!(g.stretch, Stretch::None);
        assert_eq!(g.shrink.sp(), 0);
    }

    #[test]
    fn parse_plus_minus() {
        let g = Glue::parse("10pt plus 2pt minus 1pt").unwrap();
        assert_eq!(g.width.pt(), 10.0);
        assert!(matches!(g.stretch, Stretch::Finite(d) if d.pt() == 2.0));
        assert_eq!(g.shrink.pt(), 1.0);
    }

    #[test]
    fn parse_infinite_fil() {
        let g = Glue::parse("0pt plus 1fil").unwrap();
        assert!(g.is_infinite());
        assert!(matches!(
            g.stretch,
            Stretch::Infinite {
                amount: 1,
                unit: InfiniteUnit::Fil
            }
        ));
    }

    #[test]
    fn parse_infinite_fill() {
        let g = Glue::parse("5pt plus 2fill minus 1pt").unwrap();
        assert!(matches!(
            g.stretch,
            Stretch::Infinite {
                amount: 2,
                unit: InfiniteUnit::Fill
            }
        ));
    }

    #[test]
    fn hfill_preset() {
        let g = Glue::hfill();
        assert!(g.is_infinite());
        assert_eq!(g.width.sp(), 0);
    }

    #[test]
    fn resolve_finite_stretch() {
        let g = Glue::parse("10pt plus 10pt minus 0pt").unwrap();
        let at_half = g.resolve_finite(0.5);
        assert_eq!(at_half.pt(), 15.0);
        let at_full = g.resolve_finite(1.0);
        assert_eq!(at_full.pt(), 20.0);
    }

    #[test]
    fn infinite_unit_ranking() {
        assert!(InfiniteUnit::Fil < InfiniteUnit::Fill);
        assert!(InfiniteUnit::Fill < InfiniteUnit::Filll);
    }

    #[test]
    fn pt_to_sp_constant() {
        assert_eq!(PT_TO_SP, 65536);
    }
}
