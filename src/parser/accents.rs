//! Text accent commands (`\'e`, `\"{u}`, `\c{c}`, ...) → composed Unicode.
//!
//! LaTeX writes accented letters as an accent command plus a base
//! letter, either attached (`\'e`), braced (`\'{e}`), or as a whole
//! group (`\c{c}`). This module composes accent + base into a single
//! precomposed character where Unicode has one (Latin-1/Latin
//! Extended-A) and falls back to base + combining mark otherwise.

/// Combining mark for each accent command character.
fn combining_char(accent: char) -> Option<char> {
    Some(match accent {
        '\'' => '\u{0301}', // acute
        '`' => '\u{0300}',  // grave
        '^' => '\u{0302}',  // circumflex
        '"' => '\u{0308}',  // diaeresis
        '~' => '\u{0303}',  // tilde
        '=' => '\u{0304}',  // macron
        '.' => '\u{0307}',  // dot above
        'u' => '\u{0306}',  // breve
        'v' => '\u{030C}',  // caron
        'H' => '\u{030B}',  // double acute
        'c' => '\u{0327}',  // cedilla
        'k' => '\u{0328}',  // ogonek
        'r' => '\u{030A}',  // ring above
        'b' => '\u{0331}',  // macron below
        'd' => '\u{0323}',  // dot below
        't' => '\u{0361}',  // tie
        _ => return None,
    })
}

/// Precomposed character for (accent, lowercase base), if Unicode has
/// one. Uppercase bases are derived by uppercasing the result.
fn precomposed_lower(accent: char, base: char) -> Option<char> {
    let c = match (accent, base) {
        // Acute
        ('\'', 'a') => 'á',
        ('\'', 'c') => 'ć',
        ('\'', 'e') => 'é',
        ('\'', 'g') => 'ǵ',
        ('\'', 'i') => 'í',
        ('\'', 'l') => 'ĺ',
        ('\'', 'n') => 'ń',
        ('\'', 'o') => 'ó',
        ('\'', 'r') => 'ŕ',
        ('\'', 's') => 'ś',
        ('\'', 'u') => 'ú',
        ('\'', 'y') => 'ý',
        ('\'', 'z') => 'ź',
        // Grave
        ('`', 'a') => 'à',
        ('`', 'e') => 'è',
        ('`', 'i') => 'ì',
        ('`', 'o') => 'ò',
        ('`', 'u') => 'ù',
        ('`', 'n') => 'ǹ',
        ('`', 'w') => 'ẁ',
        ('`', 'y') => 'ỳ',
        // Circumflex
        ('^', 'a') => 'â',
        ('^', 'c') => 'ĉ',
        ('^', 'e') => 'ê',
        ('^', 'g') => 'ĝ',
        ('^', 'h') => 'ĥ',
        ('^', 'i') => 'î',
        ('^', 'j') => 'ĵ',
        ('^', 'o') => 'ô',
        ('^', 's') => 'ŝ',
        ('^', 'u') => 'û',
        ('^', 'w') => 'ŵ',
        ('^', 'y') => 'ŷ',
        ('^', 'z') => 'ẑ',
        // Diaeresis
        ('"', 'a') => 'ä',
        ('"', 'e') => 'ë',
        ('"', 'h') => 'ḧ',
        ('"', 'i') => 'ï',
        ('"', 'o') => 'ö',
        ('"', 't') => 'ẗ',
        ('"', 'u') => 'ü',
        ('"', 'w') => 'ẅ',
        ('"', 'x') => 'ẍ',
        ('"', 'y') => 'ÿ',
        // Tilde
        ('~', 'a') => 'ã',
        ('~', 'e') => 'ẽ',
        ('~', 'i') => 'ĩ',
        ('~', 'n') => 'ñ',
        ('~', 'o') => 'õ',
        ('~', 'u') => 'ũ',
        ('~', 'v') => 'ṽ',
        ('~', 'y') => 'ỹ',
        // Macron
        ('=', 'a') => 'ā',
        ('=', 'e') => 'ē',
        ('=', 'g') => 'ḡ',
        ('=', 'i') => 'ī',
        ('=', 'o') => 'ō',
        ('=', 'u') => 'ū',
        ('=', 'y') => 'ȳ',
        // Dot above
        ('.', 'b') => 'ḃ',
        ('.', 'c') => 'ċ',
        ('.', 'd') => 'ḋ',
        ('.', 'e') => 'ė',
        ('.', 'f') => 'ḟ',
        ('.', 'g') => 'ġ',
        ('.', 'h') => 'ḣ',
        ('.', 'i') => 'i', // dotless-ı + dot = dotted i
        ('.', 'm') => 'ṁ',
        ('.', 'n') => 'ṅ',
        ('.', 'o') => 'ȯ',
        ('.', 'p') => 'ṗ',
        ('.', 's') => 'ṡ',
        ('.', 't') => 'ṫ',
        ('.', 'w') => 'ẇ',
        ('.', 'x') => 'ẋ',
        ('.', 'y') => 'ẏ',
        ('.', 'z') => 'ż',
        // Caron
        ('v', 'a') => 'ǎ',
        ('v', 'c') => 'č',
        ('v', 'd') => 'ď',
        ('v', 'e') => 'ě',
        ('v', 'g') => 'ǧ',
        ('v', 'h') => 'ȟ',
        ('v', 'i') => 'ǐ',
        ('v', 'j') => 'ǰ',
        ('v', 'k') => 'ǩ',
        ('v', 'l') => 'ľ',
        ('v', 'n') => 'ň',
        ('v', 'o') => 'ǒ',
        ('v', 'r') => 'ř',
        ('v', 's') => 'š',
        ('v', 't') => 'ť',
        ('v', 'u') => 'ǔ',
        ('v', 'z') => 'ž',
        // Breve
        ('u', 'a') => 'ă',
        ('u', 'e') => 'ĕ',
        ('u', 'g') => 'ğ',
        ('u', 'i') => 'ĭ',
        ('u', 'o') => 'ŏ',
        ('u', 'u') => 'ŭ',
        // Double acute
        ('H', 'o') => 'ő',
        ('H', 'u') => 'ű',
        ('H', 'y') => 'ӳ',
        // Cedilla
        ('c', 'c') => 'ç',
        ('c', 'd') => 'ḑ',
        ('c', 'e') => 'ȩ',
        ('c', 'g') => 'ģ',
        ('c', 'h') => 'ḩ',
        ('c', 'k') => 'ķ',
        ('c', 'l') => 'ļ',
        ('c', 'n') => 'ņ',
        ('c', 'r') => 'ŗ',
        ('c', 's') => 'ş',
        ('c', 't') => 'ţ',
        // Ogonek
        ('k', 'a') => 'ą',
        ('k', 'e') => 'ę',
        ('k', 'i') => 'į',
        ('k', 'o') => 'ǫ',
        ('k', 'u') => 'ų',
        // Ring above
        ('r', 'a') => 'å',
        ('r', 'u') => 'ů',
        ('r', 'w') => 'ẘ',
        ('r', 'y') => 'ẙ',
        _ => return None,
    };
    Some(c)
}

/// Compose `accent command + base letter` into output text. Returns
/// the precomposed character when Unicode has one, otherwise the base
/// letter followed by the combining mark. Uppercase bases are derived
/// from the lowercase table.
pub fn compose_accent(accent: char, base: char) -> String {
    let lower = base.to_ascii_lowercase();
    if let Some(c) = precomposed_lower(accent, lower) {
        if base.is_uppercase() {
            return c.to_uppercase().collect();
        }
        return c.to_string();
    }
    match combining_char(accent) {
        Some(mark) => {
            let mut s = String::with_capacity(3);
            s.push(base);
            s.push(mark);
            s
        }
        None => base.to_string(),
    }
}

/// The accent command characters, as TeX control-sequence names.
/// Multi-letter names never occur: accent commands are exactly one
/// character (`'`, `` ` ``, `"`, `^`, `~`, `=`, `.`, `u`, `v`, `H`,
/// `c`, `k`, `r`, `b`, `d`, `t`).
pub fn is_accent_command(name: &str) -> bool {
    matches!(
        name,
        "'" | "`" | "\"" | "^" | "~" | "=" | "." | "u" | "v" | "H" | "c" | "k" | "r" | "b" | "d"
            | "t"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_common_latin1_accents() {
        assert_eq!(compose_accent('"', 'u'), "ü");
        assert_eq!(compose_accent('\'', 'e'), "é");
        assert_eq!(compose_accent('`', 'a'), "à");
        assert_eq!(compose_accent('^', 'o'), "ô");
        assert_eq!(compose_accent('~', 'n'), "ñ");
        assert_eq!(compose_accent('c', 'c'), "ç");
    }

    #[test]
    fn composes_latin_extended_accents() {
        assert_eq!(compose_accent('v', 's'), "š");
        assert_eq!(compose_accent('v', 'z'), "ž");
        assert_eq!(compose_accent('v', 'c'), "č");
        assert_eq!(compose_accent('u', 'g'), "ğ");
        assert_eq!(compose_accent('H', 'o'), "ő");
        assert_eq!(compose_accent('=', 'a'), "ā");
        assert_eq!(compose_accent('k', 'a'), "ą");
        assert_eq!(compose_accent('r', 'u'), "ů");
    }

    #[test]
    fn uppercases_derived_from_lowercase_table() {
        assert_eq!(compose_accent('"', 'U'), "Ü");
        assert_eq!(compose_accent('\'', 'E'), "É");
        assert_eq!(compose_accent('v', 'S'), "Š");
        assert_eq!(compose_accent('c', 'C'), "Ç");
    }

    #[test]
    fn falls_back_to_combining_marks() {
        // No precomposed form for these combos: base + combining mark.
        assert_eq!(compose_accent('d', 'h'), "h\u{0323}");
        assert_eq!(compose_accent('b', 'd'), "d\u{0331}");
        assert_eq!(compose_accent('t', 's'), "s\u{0361}");
    }

    #[test]
    fn dotless_i_dots_to_i() {
        // `\.{i}` produces a normal dotted i.
        assert_eq!(compose_accent('.', 'i'), "i");
    }

    #[test]
    fn accent_command_names() {
        assert!(is_accent_command("'"));
        assert!(is_accent_command("`"));
        assert!(is_accent_command("\""));
        assert!(is_accent_command("^"));
        assert!(is_accent_command("~"));
        assert!(is_accent_command("="));
        assert!(is_accent_command("."));
        for name in ["u", "v", "H", "c", "k", "r", "b", "d", "t"] {
            assert!(is_accent_command(name), "{name}");
        }
        // Longer names are commands, not accents.
        assert!(!is_accent_command("cline"));
        assert!(!is_accent_command("textbf"));
        assert!(!is_accent_command("today"));
        assert!(!is_accent_command("underbrace"));
    }
}
