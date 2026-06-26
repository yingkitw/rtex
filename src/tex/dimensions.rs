//! TeX dimension system.
//!
//! Parses strings such as `12pt`, `-3.5mm`, `1in`, `2em`, `1ex`
//! into absolute values expressed in *scaled points* (sp), the
//! smallest unit TeX understands.

/// 1 pt = 65536 sp (TeX's fixed-point scaling factor).
pub const PT_TO_SP: i64 = 65536;

/// A TeX dimension, stored internally as scaled points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dimension {
    sp: i64,
}

impl Dimension {
    /// Create from a raw scaled-point value.
    pub fn from_sp(sp: i64) -> Self {
        Self { sp }
    }

    /// Raw scaled-point value.
    pub fn sp(&self) -> i64 {
        self.sp
    }

    /// Convert to points as a floating-point value.
    pub fn pt(&self) -> f64 {
        self.sp as f64 / PT_TO_SP as f64
    }

    /// Parse a dimension string.
    ///
    /// Supported units: `pt`, `mm`, `cm`, `in`, `bp`, `dd`, `cc`,
    /// `sp`, `em`, `ex`, `pc`, `nd`, `nc`.
    ///
    /// `em` and `ex` are relative to the current font; they are
    /// parsed but stored using a default 10 pt approximation.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        let (numeric, unit) = s.split_at(
            s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                .unwrap_or(s.len()),
        );

        let value: f64 = numeric.parse().ok()?;
        let unit = unit.trim();

        let sp = match unit {
            "pt" => (value * PT_TO_SP as f64).round() as i64,
            "mm" => (value * 1864679.81).round() as i64, // 1 mm ≈ 1864679.81 sp
            "cm" => (value * 18646798.1).round() as i64,
            "in" => (value * 4736286.72).round() as i64,
            "bp" => (value * 65536.0 * 72.0 / 72.27).round() as i64,
            "pc" => (value * 12.0 * PT_TO_SP as f64).round() as i64,
            "dd" => (value * 1238.0 * PT_TO_SP as f64 / 1157.0).round() as i64,
            "cc" => (value * 14856.0 * PT_TO_SP as f64 / 1157.0).round() as i64,
            "sp" => value.round() as i64,
            "nd" => (value * 1238.0 * PT_TO_SP as f64 / 1157.0).round() as i64,
            "nc" => (value * 14856.0 * PT_TO_SP as f64 / 1157.0).round() as i64,
            "em" => (value * 10.0 * PT_TO_SP as f64).round() as i64, // approx
            "ex" => (value * 4.3 * PT_TO_SP as f64).round() as i64, // approx
            _ => return None,
        };

        Some(Self { sp })
    }

    /// Add two dimensions.
    pub fn plus(self, other: Self) -> Self {
        Self::from_sp(self.sp + other.sp)
    }

    /// Subtract two dimensions.
    pub fn minus(self, other: Self) -> Self {
        Self::from_sp(self.sp - other.sp)
    }

    /// Scale by a factor.
    pub fn scale(self, factor: f64) -> Self {
        Self::from_sp((self.sp as f64 * factor).round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pt() {
        let d = Dimension::parse("12pt").unwrap();
        assert_eq!(d.sp(), 12 * PT_TO_SP);
    }

    #[test]
    fn parse_negative() {
        let d = Dimension::parse("-3.5pt").unwrap();
        assert_eq!(d.pt(), -3.5);
    }

    #[test]
    fn parse_mm() {
        let d = Dimension::parse("10mm").unwrap();
        assert!(d.sp() > 0);
    }

    #[test]
    fn parse_in() {
        let d = Dimension::parse("1in").unwrap();
        // 1 in = 72.27 pt
        assert!((d.pt() - 72.27).abs() < 0.01);
    }

    #[test]
    fn parse_em_approx() {
        let d = Dimension::parse("1em").unwrap();
        // approx 10 pt
        assert_eq!(d.sp(), (10.0 * PT_TO_SP as f64).round() as i64);
    }

    #[test]
    fn invalid_unit() {
        assert!(Dimension::parse("12px").is_none());
    }

    #[test]
    fn arithmetic() {
        let a = Dimension::parse("10pt").unwrap();
        let b = Dimension::parse("3pt").unwrap();
        assert_eq!(a.plus(b).pt(), 13.0);
        assert_eq!(a.minus(b).pt(), 7.0);
        assert_eq!(a.scale(2.0).pt(), 20.0);
    }
}
