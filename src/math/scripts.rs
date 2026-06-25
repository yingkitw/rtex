/// Convert a base character to its Unicode superscript equivalent.
pub fn to_superscript(ch: char) -> Option<char> {
    match ch {
        '0' => Some('⁰'),
        '1' => Some('¹'),
        '2' => Some('²'),
        '3' => Some('³'),
        '4' => Some('⁴'),
        '5' => Some('⁵'),
        '6' => Some('⁶'),
        '7' => Some('⁷'),
        '8' => Some('⁸'),
        '9' => Some('⁹'),
        '+' => Some('⁺'),
        '-' => Some('⁻'),
        '=' => Some('⁼'),
        '(' => Some('⁽'),
        ')' => Some('⁾'),
        'a' => Some('ᵃ'),
        'b' => Some('ᵇ'),
        'c' => Some('ᶜ'),
        'd' => Some('ᵈ'),
        'e' => Some('ᵉ'),
        'f' => Some('ᶠ'),
        'g' => Some('ᵍ'),
        'h' => Some('ʰ'),
        'i' => Some('ⁱ'),
        'j' => Some('ʲ'),
        'k' => Some('ᵏ'),
        'l' => Some('ˡ'),
        'm' => Some('ᵐ'),
        'n' => Some('ⁿ'),
        'o' => Some('ᵒ'),
        'p' => Some('ᵖ'),
        'r' => Some('ʳ'),
        's' => Some('ˢ'),
        't' => Some('ᵗ'),
        'u' => Some('ᵘ'),
        'v' => Some('ᵛ'),
        'w' => Some('ʷ'),
        'x' => Some('ˣ'),
        'y' => Some('ʸ'),
        'z' => Some('ᶻ'),
        _ => None,
    }
}

/// Convert a base character to its Unicode subscript equivalent.
pub fn to_subscript(ch: char) -> Option<char> {
    match ch {
        '0' => Some('₀'),
        '1' => Some('₁'),
        '2' => Some('₂'),
        '3' => Some('₃'),
        '4' => Some('₄'),
        '5' => Some('₅'),
        '6' => Some('₆'),
        '7' => Some('₇'),
        '8' => Some('₈'),
        '9' => Some('₉'),
        '+' => Some('₊'),
        '-' => Some('₋'),
        '=' => Some('₌'),
        '(' => Some('₍'),
        ')' => Some('₎'),
        'a' => Some('ₐ'),
        'e' => Some('ₑ'),
        'h' => Some('ₕ'),
        'i' => Some('ᵢ'),
        'j' => Some('ⱼ'),
        'k' => Some('ₖ'),
        'l' => Some('ₗ'),
        'm' => Some('ₘ'),
        'n' => Some('ₙ'),
        'o' => Some('ₒ'),
        'p' => Some('ₚ'),
        'r' => Some('ᵣ'),
        's' => Some('ₛ'),
        't' => Some('ₜ'),
        'u' => Some('ᵤ'),
        'v' => Some('ᵥ'),
        'x' => Some('ₓ'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superscript_digits() {
        assert_eq!(to_superscript('0'), Some('⁰'));
        assert_eq!(to_superscript('1'), Some('¹'));
        assert_eq!(to_superscript('5'), Some('⁵'));
        assert_eq!(to_superscript('9'), Some('⁹'));
    }

    #[test]
    fn test_superscript_letters() {
        assert_eq!(to_superscript('a'), Some('ᵃ'));
        assert_eq!(to_superscript('n'), Some('ⁿ'));
        assert_eq!(to_superscript('x'), Some('ˣ'));
    }

    #[test]
    fn test_superscript_punctuation() {
        assert_eq!(to_superscript('+'), Some('⁺'));
        assert_eq!(to_superscript('-'), Some('⁻'));
        assert_eq!(to_superscript('('), Some('⁽'));
    }

    #[test]
    fn test_superscript_unmapped_returns_none() {
        assert_eq!(to_superscript('q'), None);
        assert_eq!(to_superscript('A'), None);
    }

    #[test]
    fn test_subscript_digits() {
        assert_eq!(to_subscript('0'), Some('₀'));
        assert_eq!(to_subscript('1'), Some('₁'));
        assert_eq!(to_subscript('5'), Some('₅'));
    }

    #[test]
    fn test_subscript_letters() {
        assert_eq!(to_subscript('a'), Some('ₐ'));
        assert_eq!(to_subscript('i'), Some('ᵢ'));
        assert_eq!(to_subscript('x'), Some('ₓ'));
    }

    #[test]
    fn test_subscript_unmapped_returns_none() {
        assert_eq!(to_subscript('b'), None);
        assert_eq!(to_subscript('c'), None);
        assert_eq!(to_subscript('Z'), None);
    }
}
